mod db;
mod game;
mod site;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Query, State,
    },
    // http::{Response, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    pub redis_conn_mgr: ConnectionManager,
}

#[tokio::main]
async fn main() {
    let redis_address = std::env::var("REDIS_ADDRESS").unwrap();
    let client = redis::Client::open(redis_address).unwrap();
    let redis_conn_mgr = client.get_tokio_connection_manager().await.unwrap();

    let shared_state = Arc::new(AppState {
        redis_conn_mgr: redis_conn_mgr,
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

async fn handle_socket(mut socket: WebSocket, join_token: Option<String>, _state: Arc<AppState>) {
    // let redis = state.redis_conn_mgr.clone();
    println!(
        "New WebSocket connection with join token: '{:?}'",
        join_token
    );

    // if join token blank, make new one and associate it with channel
    // if join token present, hook this new socket to the game channel somehow
    // if join token present but invalid, just make a new game with a new token

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                println!("Received: {:?}", msg);
                // echo the message back
                if socket.send(msg).await.is_err() {
                    println!("Error sending message, client disconnected");
                    return;
                }

                // let msg = msg.to_str().unwrap();
                // let msg = msg.split(" ").collect::<Vec<&str>>();
                // let cmd = msg[0];
                // let args = msg[1..].to_vec();

                // match cmd {
                //     "get" => {
                //         let key = args[0];
                //         let value = db::get(redis.clone(), key).await;
                //         socket.send(value).await.unwrap();
                //     }
                //     "set" => {
                //         let key = args[0];
                //         let value = args[1];
                //         db::set(redis.clone(), key, value).await;
                //     }
                //     _ => {
                //         println!("Unknown command: {}", cmd);
                //     }
                // }
            }
            Err(e) => {
                println!("Error, client disconnected: {:?}", e);
                return;
            }
        }
    }
}
