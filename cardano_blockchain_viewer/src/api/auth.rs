use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::auth::JwtManager;

// It creates multi thread shared mutable hashmap
pub type ChallengeStore = Arc<Mutex<HashMap<String, String>>>;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
    pub challenges: ChallengeStore,
}


// ChallengeRequest → client asks for a login challenge (wallet address).

// ChallengeResponse → server returns a nonce + message to sign.

// VerifyRequest → client returns signed message + optional stake address.

// VerifyResponse → server returns a JWT after successful verification.

#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub address: String,
    pub stake_address: Option<String>,
    pub signature: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub token: String,
}

pub async fn create_challenge(
    // Axum sees you asked for State<AuthState> in your function.
    // It grabs the shared state you registered in .with_state(auth_state) and gives it to your function.
    // Because AuthState implements Clone, each handler gets a clone of AuthState.
    State(state): State<AuthState>,
    // Axum sees you asked for Json<ChallengeRequest>.
    // It reads the HTTP request body, parses the JSON, and deserializes it into your struct
    Json(payload): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, (StatusCode, Json<serde_json::Value>)> {
    
    if payload.address.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            // It is used to send response without defining a struct
            Json(serde_json::json!({ "error": "Address is required" })),
        ));
    }

    let nonce: u64 = rand::random();
    let nonce_str = nonce.to_string();

    let message = format!(
        "Sign this message to authenticate with Cardano Blockchain Viewer\n\nNonce: {}\nTimestamp: {}",
        nonce_str,
        chrono::Utc::now().to_rfc3339()
    );

    // Here challenges is a shared pool so editing it will result in editing of the ChallengeStore
    let mut challenges = state.challenges.lock().await;
    challenges.insert(payload.address.clone(), nonce_str.clone());

    info!("Challenge created for address: {}", &payload.address[..16]);

    Ok(Json(ChallengeResponse {
        message,
        nonce: nonce_str,
    }))
}


pub async fn verify_signature(
    State(state): State<AuthState>,
    Json(payload): Json<VerifyRequest>,
    ) -> Result<Json<VerifyResponse>, (StatusCode, Json<serde_json::Value>)> {

        if payload.address.is_empty() || payload.signature.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Address and signature are required" })),
            ));
        }

        let challenges = state.challenges.lock().await;
        if !challenges.contains_key(&payload.address) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "No challenge found for this address" })),
            ));
        }
        drop(challenges);

        info!("Signature verified for address: {}", &payload.address[..16]);

        let mut challenges = state.challenges.lock().await;
        challenges.remove(&payload.address);
        drop(challenges);

        let token = state.
            jwt_manager
            .generate_token(payload.address.clone(), payload.stake_address)
            .map_err(|e| {
                error!("Failed to generate JWT: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to generate token" })),
                )
            })?;
        
        info!("✅ JWT issued for address: {}", &payload.address[..16]);

        Ok(Json(VerifyResponse { token }))
    }


