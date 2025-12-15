use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

use crate::models::AppState;

// Handle a new WebSocket connection from a client
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<AppState>>,
    mut rx: broadcast::Receiver<String>,
) {
    info!("New WebSocket connection from: {}", addr);

    // Accept WebSocket connection
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake error: {}", e);
            return;
        }
    };

    // Splitting to sender and reciever
    // ws_sender -> sends message to the client
    // ws_receiver -> receives message from the client
    let (mut ws_sender, mut ws_reciever) = ws_stream.split();

    // Send current buffer to new client
    {
        let state = state.lock().await;
        let stats = state.get_stats();

        // Send stats first
        let stats_msg = serde_json::json!({
            "type": "stats",
            "data": stats
        });
        if let Ok(msg) = serde_json::to_string(&stats_msg){
            let _ = ws_sender.send(Message::Text(msg)).await;
        }

        // Send buffered events
        for event in &state.buffer {
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = ws_sender.send(Message::Text(json)).await;
            }
        }
    }

    // Spawn task to send broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err(){
                break;
            }
        }
    });


    // Handle incoming messages (ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_reciever.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(_)) => {
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