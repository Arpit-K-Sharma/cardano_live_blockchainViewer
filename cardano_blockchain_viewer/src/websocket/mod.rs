// WebSocket module - handles client connections

pub mod axum_handler;
pub mod handler;

pub use axum_handler::{websocket_handler, WebSocketState};
