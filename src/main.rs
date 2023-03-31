mod db;
mod game;
mod site;

use crate::game::Game;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    // http::{Response, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone)]
struct AppState {
    pub redis_conn_mgr: ConnectionManager,
    pub games: Arc<Mutex<HashMap<String, Arc<Mutex<Game>>>>>,
}

#[tokio::main]
async fn main() {
    let redis_address = std::env::var("REDIS_ADDRESS").unwrap();
    let client = redis::Client::open(redis_address).unwrap();
    let redis_conn_mgr = client.get_tokio_connection_manager().await.unwrap();

    let shared_state = Arc::new(AppState {
        redis_conn_mgr: redis_conn_mgr,
        games: Arc::new(Mutex::new(HashMap::new())),
        // sockets: Vec::new(),
    });

    let app = Router::new()
        .route("/", get(site::index))
        .route("/ws", get(open_conn))
        .fallback(get(site::static_file_server))
        .with_state(shared_state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct NewGameParams {
    #[serde(default)]
    pub token: Option<String>,
}

async fn open_conn(
    Query(params): Query<NewGameParams>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> Response {
    let join_token = params
        .token
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    ws.on_upgrade(|socket| handle_socket(socket, join_token, state))
}

async fn handle_socket(socket: WebSocket, join_token: Option<String>, state: Arc<AppState>) {
    // let redis = state.redis_conn_mgr.clone();
    println!(
        "New WebSocket connection with join token: '{:?}'",
        join_token
    );

    let game: Arc<Mutex<Game>>;
    let mut receive_from_game: watch::Receiver<game::State>;

    match join_token {
        Some(token) => {
            // find the game and connect to it
            let games = state.games.lock().unwrap();
            game = games.get(&token).unwrap().clone();
            let game = game.lock().unwrap();
            receive_from_game = game.state_changes.subscribe();
        }
        None => {
            // create a new game
            let id = random_token();
            let (tx, rx) = watch::channel(game::State::new());
            receive_from_game = rx;
            game = Arc::new(Mutex::new(Game::new(id.clone(), tx)));
            state.games.lock().unwrap().insert(id, game.clone());
        }
    };

    let (mut send_to_web, mut recv_from_web) = socket.split();

    loop {
        tokio::select! {
            _ = receive_from_game.changed() => {
                let new_state = receive_from_game.borrow().clone();
                println!("Socket: Received game state change: {:?}", new_state);
                let json = serde_json::to_string(&new_state).unwrap();
                send_to_web.send(Message::Text(json)).await.unwrap();
            }
            msg = recv_from_web.next() => {
                match msg {
                    Some(msg) => {
                        println!("Socket: Received message: {:?}", msg);
                        match msg {
                            Ok(Message::Text(msg)) => {
                                game.lock().unwrap().handle_msg(game::StateChange::ChatMsg(msg)).unwrap();
                            }

                            Ok(Message::Close(_)) => {
                                println!("Socket: Client closed connection");
                                return;
                            }

                            _ => {
                                println!("Socket: Unhandled message type");
                            }
                        }
                    }
                    None => {
                        println!("Socket: Client disconnected");
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
