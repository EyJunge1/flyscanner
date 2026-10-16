use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

use super::routes::AppState;
use crate::icao::Icao24;
use crate::state::Aircraft;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Full fleet replace (connect + periodic resync).
    Snapshot {
        aircraft: Vec<Aircraft>,
    },
    /// Legacy full-fleet push (kept for older clients).
    Update {
        aircraft: Vec<Aircraft>,
    },
    /// Incremental: upsert changed aircraft, remove expired ICAOs.
    Delta {
        upsert: Vec<Aircraft>,
        remove: Vec<Icao24>,
    },
    Stats {
        messages_total: u64,
        aircraft_current: i64,
        message_rate: f64,
        max_range_nm: f64,
    },
    Pong,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Initial snapshot
    {
        let store = state.store.read().await;
        let aircraft = store.list_owned();
        if let Ok(json) = serde_json::to_string(&WsMessage::Snapshot { aircraft }) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg)
                && sender.send(Message::Text(json.into())).await.is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(t) if t == "ping" => {
                    debug!("ws ping");
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
