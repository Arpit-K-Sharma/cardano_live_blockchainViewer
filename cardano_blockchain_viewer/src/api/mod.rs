// src/api/mod.rs
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub mod auth;
pub mod user;

use crate::auth::{auth_middleware, JwtManager};
use crate::blockfrost::BlockfrostClient;
use crate::websocket::{websocket_handler, WebSocketState};

pub fn create_router(
    jwt_manager: Arc<JwtManager>,
    blockfrost: Arc<BlockfrostClient>,
    ws_state: WebSocketState,
) -> Router {
    let auth_state = auth::AuthState {
        jwt_manager: jwt_manager.clone(),
        challenges: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    };

    let user_state = user::UserState { blockfrost };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/api/auth/challenge", post(auth::create_challenge))
        .route("/api/auth/verify", post(auth::verify_signature))
        .with_state(auth_state);

    let protected_routes = Router::new()
        .route("/api/user/transactions", get(user::get_transactions))
        .route("/api/user/summary", get(user::get_summary))
        .with_state(user_state)
        .layer(middleware::from_fn_with_state(
            jwt_manager,
            auth_middleware,
        ));

    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(ws_state)
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}