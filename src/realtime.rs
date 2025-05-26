use std::sync::OnceLock;

use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message};
use axum::{extract::State, response::IntoResponse};
use futures_util::StreamExt;
use tokio::sync::broadcast::{self, Sender};
use serde::{Deserialize};
use crate::{handlers, SharedState};

static BROADCASTER: OnceLock<Sender<(String, String)>> = OnceLock::new();

pub fn init_broadcaster() {
    let (tx, _) = broadcast::channel::<(String, String)>(100);
    BROADCASTER.set(tx).unwrap();
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WsCommand {
    Start,
    Stop,
    Kill,
    Template { id: u32 },
}

pub async fn handle_ws(mut socket: WebSocket, state: SharedState) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            let reply = match serde_json::from_str::<WsCommand>(&text) {
                Ok(cmd) => handlers::handle_ws_command(cmd,state.clone()),
                Err(_) => "Ukendt eller ugyldig kommando".to_string(),
            };

            let _ = socket.send(Message::Text(reply)).await;
        }
    }
}

pub fn notify_session(session_id: &str, message: &str) {
    let _ = BROADCASTER.get().unwrap().send((session_id.to_string(), message.to_string()));
}