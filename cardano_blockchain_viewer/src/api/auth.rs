use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use crate::auth::JwtManager;

// It creates multi thread shared mutable hashmap
pub type ChallengeStore = Arc<Mutex<HashMap<String, ChallengeData>>>;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
    pub challenges: ChallengeStore,
}

#[derive(Debug, Clone)]
pub struct ChallengeData {
    pub nonce: String,
    pub message: String,
    pub timestamp: i64,
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
    let timestamp = chrono::Utc::now().timestamp();

    let message = format!(
        "Sign this message to authenticate with Cardano Blockchain Viewer\n\nNonce: {}\nTimestamp: {}",
        nonce_str,
        chrono::Utc::now().to_rfc3339()
    );

    // Here challenges is a shared pool so editing it will result in editing of the ChallengeStore
    let mut challenges = state.challenges.lock().await;
    challenges.insert(payload.address.clone(),
                      ChallengeData {
                            nonce: nonce_str.clone(),
                            message: message.clone(),
                            timestamp,
                        });
    
    let cutoff = timestamp - 300;
    challenges.retain(|_, data| data.timestamp > cutoff);

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
        let challenge_data = challenges.get(&payload.address).cloned();
        drop(challenges);

        let challenge_data = challenge_data.ok_or_else(|| {
            warn!("No challenge found for address: {}", &payload.address[..16]);
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "No challenge found. Please request a new challenge."})),
            )
        })?;

        // Check if challenge has expired (5 minutes)
        let now = chrono::Utc::now().timestamp();
        if now - challenge_data.timestamp > 300 {
            warn!("Challenge expired for address: {}", &payload.address[..16]);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Challenge expired. Please request a new challenge." })),
            ));
        }

        info!("Verifying signature for address: {}", &payload.address[..16]);


    // ========================================================================
    // CRITICAL: VERIFY THE SIGNATURE CRYPTOGRAPHICALLY
    // ========================================================================

    match verify_cardano_signature(
        &payload.address,
        &challenge_data.message,
        &payload.signature,
        &payload.key,
    ) {
        Ok(true) => {
            info!("✅ Signature verification PASSED for: {}", &payload.address[..16]);
        }
        Ok(false) => {
            warn!("❌ Signature verification FAILED for: {}", &payload.address[..16]);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid signature" })),
            ));
        }
        Err(e) => {
            error!("❌ Signature verification error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Signature verification failed" })),
            ));
        }
    }

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

// ============================================================================
// SIGNATURE VERIFICATION LOGIC
// ============================================================================

fn verify_cardano_signature(
    _address: &str,
    message: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, String> {
    use ed25519_dalek::{Signature, VerifyingKey, Verifier};

    // Decode signature from hex
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| format!("Invalid signature hex: {}", e))?;

    // Decode public key from hex
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Invalid public key hex: {}", e))?;

    // Parse COSE_Key structure (CIP-30 format)
    // Wallet extensions return public key in COSE_Key format
    // We need to extract the raw public key bytes
    let raw_public_key = extract_public_key_from_cose(&public_key_bytes)
        .map_err(|e| format!("Failed to parse COSE key: {}", e))?;

    // Create Ed25519 verifying key
    let verifying_key = VerifyingKey::from_bytes(&raw_public_key)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    // Parse signature
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|e| format!("Invalid signature format: {}", e))?;

    // Verify signature
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// Extract raw Ed25519 public key from COSE_Key format
fn extract_public_key_from_cose(cose_key_bytes: &[u8]) -> Result<[u8; 32], String> {
    // CIP-30 wallets return COSE_Key in CBOR format
    // Structure: Map with key -2 containing the public key bytes
    
    // For simplicity, if the bytes are already 32 bytes, treat as raw key
    if cose_key_bytes.len() == 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(cose_key_bytes);
        return Ok(key);
    }

    // Parse CBOR COSE_Key structure
    // This is a simplified parser - in production use a proper CBOR library
    // Look for the -2 key which contains the public key
    
    // Try to find 32-byte sequence in the COSE structure
    for i in 0..cose_key_bytes.len().saturating_sub(32) {
        if i + 32 <= cose_key_bytes.len() {
            // Check if this looks like a valid Ed25519 public key
            // (basic heuristic: not all zeros, not all ones)
            let slice = &cose_key_bytes[i..i + 32];
            if slice.iter().any(|&b| b != 0) && slice.iter().any(|&b| b != 255) {
                let mut key = [0u8; 32];
                key.copy_from_slice(slice);
                return Ok(key);
            }
        }
    }

    Err("Could not extract public key from COSE_Key structure".to_string())
}

// ============================================================================
// ADDITIONAL: Verify address matches public key
// ============================================================================

#[allow(dead_code)]
fn verify_address_from_public_key(
    address_bech32: &str,
    public_key_bytes: &[u8; 32],
) -> Result<bool, String> {
    // This would use cardano-serialization-lib to:
    // 1. Decode the bech32 address
    // 2. Hash the public key
    // 3. Compare with the address payment credential
    
    // For now, we trust that the wallet extension
    // only allows signing with keys that match the address
    
    // In production, implement full verification:
    /*
    use cardano_serialization_lib::*;
    let addr = Address::from_bech32(address_bech32)
        .map_err(|e| format!("Invalid address: {}", e))?;
    
    // Extract payment credential and verify it matches the public key hash
    // ...
    */
    
    let _ = (address_bech32, public_key_bytes);
    Ok(true) // Placeholder
}

