use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, broadcast};
use tracing::{error, info};

mod config;
mod models;
mod services;
mod websocket;
mod blockfrost;
mod auth;
mod api;

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


    let jwt_secret = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| {
                        tracing::warn!(" ⚠️  JWT_SECRET not set, using default (CHANGE IN PRODUCTION!)");
            "change-this-secret-in-production-use-strong-key".to_string()
                    });

    let blockfrost_key = std::env::var("BLOCKFROST_API_KEY")
        .expect("❌ BLOCKFROST_API_KEY environment variable must be set");

    let jwt_manager = Arc::new(auth::JwtManager::new(jwt_secret));
    let blockfrost = Arc::new(blockfrost::BlockfrostClient::new(blockfrost_key, "preprod"));
    
    info!("🔐 JWT Manager initialized");
    info!("🌐 Blockfrost client initialized (preprod network)");

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

    let api_router = api::create_router(jwt_manager, blockfrost);
    let api_addr: SocketAddr = "127.0.0.1:3001".parse()?;

    info!("🌍 REST API server starting on: http://{}", api_addr);
    info!("   - POST http://{}/api/auth/challenge", api_addr);
    info!("   - POST http://{}/api/auth/verify", api_addr);
    info!("   - GET  http://{}/api/user/transactions (protected)", api_addr);
    info!("   - GET  http://{}/api/user/summary (protected)", api_addr);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(api_addr).await.unwrap();
        axum::serve(listener, api_router).await.unwrap();
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
