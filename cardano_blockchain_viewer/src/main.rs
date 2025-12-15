use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
use tracing::{error, info};

mod config;
mod models;
mod services;
mod websocket;

use config::{BUFFER_SIZE, CardanoConfig, WEBSOCKET_ADDR};
use models::AppState;
use services::{EventProcessor, OuraReader};
use websocket::handle_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    // Get Cardano network configuration
    let cardano_config = CardanoConfig::default(); // Uses PreProd by default

    info!("Starting Cardano Blockchain Viewer Backend");
    info!("Network: {}", cardano_config.network_name);

    // Create shared application state
    let state = Arc::new(Mutex::new(AppState::new(BUFFER_SIZE)));

    // Create broadcast channels with larger capacity to handle bursts
    let (oura_tx, _) = broadcast::channel(1000); // Channel for Oura events
    let (ws_tx, _) = broadcast::channel(1000); // Channel for WebSocket broadcasts

    // Initialize services
    let oura_reader = OuraReader::new(cardano_config);
    let event_processor = EventProcessor::new(Arc::clone(&state));

    // Spawn task to read from Oura
    let oura_tx_clone = oura_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = oura_reader.start(oura_tx_clone).await {
            error!("Oura reader error: {}", e);
        }
    });

    // Spawn task to process events
    let mut oura_rx = oura_tx.subscribe();
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        while let Ok(oura_event) = oura_rx.recv().await {
            if let Err(e) = event_processor
                .process_event(oura_event, &ws_tx_clone)
                .await
            {
                error!("Event processing error: {}", e);
            }
        }
    });

    // Start WebSocket server
    let addr: SocketAddr = WEBSOCKET_ADDR.parse()?;
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on: ws://{}", addr);
    info!("Connect with: wscat -c ws://{}", addr);
    info!("Or open the HTML client in your browser");

    // Accept WebSocket connections
    while let Ok((stream, client_addr)) = listener.accept().await {
        let state = Arc::clone(&state);
        let rx = ws_tx.subscribe();
        tokio::spawn(handle_connection(stream, client_addr, state, rx));
    }

    Ok(())
}
