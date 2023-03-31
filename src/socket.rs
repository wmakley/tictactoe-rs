// types and helpers for sending and receiving game data over web sockets
use crate::game::StateChange;
use axum::extract::ws;
use serde::{Deserialize, Serialize};

/// Nice internal representation of message.
#[derive(Debug, Clone)]
pub enum OutgoingMessage {
    GameJoined(String),
    GameStateChanged(StateChange),
}

/// Ugly JSON representation of message.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct RawMessage<T> {
    msg: String,
    payload: T,
}

pub async fn send(socket: &mut ws::WebSocket, msg: &OutgoingMessage) -> Result<(), axum::Error> {
    let json = match msg {
        OutgoingMessage::GameJoined(id) => {
            let raw = RawMessage {
                msg: "game_joined".to_string(),
                payload: id,
            };
            serde_json::to_string(&raw).unwrap()
        }
        OutgoingMessage::GameStateChanged(state_change) => {
            let raw = RawMessage {
                msg: "game_state_changed".to_string(),
                payload: state_change,
            };
            serde_json::to_string(&raw).unwrap()
        }
    };
    socket.send(ws::Message::Text(json)).await
}
