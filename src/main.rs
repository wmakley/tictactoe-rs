mod db;
mod game;
mod site;

use crate::game::Game;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use rand::{distributions::Alphanumeric, Rng};
// use redis::aio::ConnectionManager;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber;

#[derive(Debug)]
struct AppState {
    // pub redis_conn_mgr: ConnectionManager,
    pub games: Arc<Mutex<HashMap<String, Arc<Mutex<Game>>>>>,
}

impl Display for AppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppState(GameCount: {})",
            self.games.lock().unwrap().len()
        )
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // let redis_address = std::env::var("REDIS_ADDRESS").unwrap();
    // let client = redis::Client::open(redis_address).unwrap();
    // let redis_conn_mgr = client.get_tokio_connection_manager().await.unwrap();

    let shared_state = Arc::new(AppState {
        // redis_conn_mgr: redis_conn_mgr,
        games: Arc::new(Mutex::new(HashMap::new())),
        // sockets: Vec::new(),
    });

    let app = Router::new()
        .route("/", get(site::index))
        .route("/ws", get(open_conn))
        .route("/health", get(|| async { StatusCode::OK }))
        .fallback(get(site::static_file_server))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:3000";
    println!("Starting server on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct NewGameParams {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

impl NewGameParams {
    pub fn normalized(&self) -> NewGameParams {
        NewGameParams {
            token: self
                .token
                .clone()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            name: self
                .name
                .clone()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        }
    }
}

async fn open_conn(
    Query(params): Query<NewGameParams>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> Response {
    let params = params.normalized();
    ws.on_upgrade(|socket| handle_socket(socket, params, state))
}

async fn handle_socket(mut socket: WebSocket, params: NewGameParams, state: Arc<AppState>) {
    // let redis = state.redis_conn_mgr.clone();
    debug!("New WebSocket connection with params: '{:?}'", params);

    let game: Arc<Mutex<Game>> = params
        .token
        .clone()
        .and_then(|token| {
            // if we have a token, try to get the game matching the token
            let games = state.games.lock().unwrap();
            games.get(&token).map(|g| g.clone())
        })
        .unwrap_or_else(|| {
            // if after that we still don't have a game, create a new one

            let id: String = params.token.unwrap_or_else(|| random_token());
            // TODO: when generating random token, check for collisions

            let (game, _) = Game::new(id.clone());

            let game = Arc::new(Mutex::new(game));

            state.games.lock().unwrap().insert(id, game.clone());
            game
        });

    // now that we got a game, add the connected user as a player,
    // extract some data from it and send state to client
    let (id, player, game_state, mut receive_from_game) = {
        let mut game = game.lock().unwrap();

        let player: game::Player;
        match game.add_player(params.name.unwrap_or_else(|| "Unnamed Player".to_string())) {
            Ok(_player) => {
                player = _player;
            }
            Err(e) => {
                error!("Socket: TODO: Error adding player: {:?}", e);
                return;
            }
        }
        game.broadcast_state();

        (
            game.id.clone(),
            player,
            game.state.clone(),
            game.state_changes.subscribe(),
        )
    };

    let json = serde_json::to_string(&game::ToBrowser::JoinedGame {
        token: id,
        team: player.team,
        state: game_state,
    })
    .unwrap();
    socket.send(Message::Text(json)).await.unwrap();

    let disconnect = || {
        debug!(
            "Socket: Player {:?} disconnected, removing from game",
            player
        );
        let mut game = game.lock().unwrap();
        game.remove_player(player.team);
        if game.state.players.is_empty() {
            debug!("Socket: Game is empty, removing globally");
            state.games.lock().unwrap().remove(&game.id);
        }
        game.broadcast_state();
    };

    loop {
        tokio::select! {
            _ = receive_from_game.changed() => {
                let new_state = receive_from_game.borrow().clone();
                // trace!("Socket: Sending game state change: {:?}", new_state);

                let json = serde_json::to_string(&game::ToBrowser::GameState(new_state)).unwrap();
                socket.send(Message::Text(json)).await.unwrap();
            }
            msg = socket.recv() => {
                match msg {
                    Some(raw_msg) => {
                        debug!("Socket: Received message: {:?}", raw_msg);
                        match raw_msg {
                            Ok(Message::Text(json)) => {
                                let parsed: game::FromBrowser = serde_json::from_str(&json).unwrap();
                                debug!("Socket: Parsed message: {:?}", parsed);

                                let server_err = {
                                    let mut game = game.lock().unwrap();
                                    let result = game.handle_msg(player.team, parsed);
                                    match result {
                                        Ok(changed) => {
                                            if changed {
                                                game.broadcast_state();
                                            }
                                            None
                                        }

                                        Err(e) => {
                                            Some(e)
                                        }
                                    }
                                    // lock game
                                };

                                if let Some(e) = server_err {
                                    debug!("Socket: Error handling message: {:?}", e);
                                    let json = serde_json::to_string(&game::ToBrowser::Error(e)).unwrap();
                                    socket.send(Message::Text(json)).await.unwrap();
                                }
                            }

                            Ok(Message::Close(_)) => {
                                debug!("Socket: Client closed connection");
                                disconnect();
                                return;
                            }

                            _ => {
                                debug!("Socket: Unhandled message type");
                            }
                        }
                    }
                    None => {
                        debug!("Socket: Client disconnected");
                        disconnect();
                        return;
                    }
                }
            }
        }
    }
}

fn random_token() -> String {
    return rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
}
