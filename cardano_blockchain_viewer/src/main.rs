use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

mod api;
mod auth;
mod blockfrost;
mod config;
mod models;
mod services;
mod websocket;

use config::{CardanoConfig, BUFFER_SIZE, SERVER_ADDR};
use models::AppState;
use services::{EventProcessor, OuraReader};
use websocket::WebSocketState;

// Health check endpoint for deployment platforms
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    // Try current working directory first, then explicitly try the backend folder
    if dotenvy::dotenv().is_err() {
        let _ = dotenvy::from_filename("cardano_blockchain_viewer/.env");
    }

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

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!(" ⚠️  JWT_SECRET not set, using default (CHANGE IN PRODUCTION!)");
        "change-this-secret-in-production-use-strong-key".to_string()
    });

    let blockfrost_key = std::env::var("BLOCKFROST_API_KEY").unwrap_or_else(|_| {
        // Attempt to load from backend-specific .env if not yet loaded
        let _ = dotenvy::from_filename("cardano_blockchain_viewer/.env");
        std::env::var("BLOCKFROST_API_KEY")
            .expect("❌ BLOCKFROST_API_KEY environment variable must be set")
    });

    let jwt_manager = Arc::new(auth::JwtManager::new(jwt_secret));
    let blockfrost_key_len = blockfrost_key.len();
    let blockfrost = Arc::new(blockfrost::BlockfrostClient::new(blockfrost_key, "preprod"));

    info!("🔐 JWT Manager initialized");
    info!("🌐 Blockfrost client initialized (preprod network)");
    info!(
        "🔑 BLOCKFROST_API_KEY loaded ({} chars)",
        blockfrost_key_len
    );

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

    // Create WebSocket state for Axum
    let ws_state = WebSocketState {
        app_state: Arc::clone(&state),
        ws_tx: ws_tx.clone(),
    };

    let api_router =
        api::create_router(jwt_manager, blockfrost, ws_state).route("/health", get(health_check));
    let server_addr: SocketAddr = SERVER_ADDR.parse()?;

    info!("🌍 Server starting on: http://{}", server_addr);
    info!("   REST API Endpoints:");
    info!("   - POST http://{}/api/auth/challenge", server_addr);
    info!("   - POST http://{}/api/auth/verify", server_addr);
    info!(
        "   - GET  http://{}/api/user/transactions (protected)",
        server_addr
    );
    info!(
        "   - GET  http://{}/api/user/summary (protected)",
        server_addr
    );
    info!("   WebSocket Endpoint:");
    info!("   - ws://{}/ws", server_addr);
    info!("   Connect with: wscat -c ws://{}/ws", server_addr);

    let listener = tokio::net::TcpListener::bind(server_addr).await?;
    axum::serve(listener, api_router).await?;

    Ok(())
}