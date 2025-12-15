use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

use crate::models::AppState;

#[derive(Clone)]
pub struct WebSocketState {
    pub app_state: Arc<Mutex<AppState>>,
    pub ws_tx: broadcast::Sender<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: WebSocketState) {
    let addr = "client"; // Axum doesn't provide peer addr in websocket upgrade
    info!("New WebSocket connection from: {}", addr);

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Send current buffer to new client
    {
        let app_state = state.app_state.lock().await;
        let stats = app_state.get_stats();

        // Send stats first
        let stats_msg = serde_json::json!({
            "type": "stats",
            "data": stats
        });
        if let Ok(msg) = serde_json::to_string(&stats_msg) {
            let _ = ws_sender
                .send(axum::extract::ws::Message::Text(msg))
                .await;
        }

        // Send buffered events
        for event in &app_state.buffer {
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = ws_sender
                    .send(axum::extract::ws::Message::Text(json))
                    .await;
            }
        }
    }

    // Spawn task to send broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender
                .send(axum::extract::ws::Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Handle incoming messages (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Close(_)) => break,
                Ok(axum::extract::ws::Message::Ping(_)) => {
                    info!("Received ping from {}", addr);
                }
                Err(e) => {
                    error!("WebSocket error from {}: {}", addr, e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed: {}", addr);
}
