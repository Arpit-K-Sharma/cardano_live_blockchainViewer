use crate::auth::JwtManager;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

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
    challenges.insert(
        payload.address.clone(),
        ChallengeData {
            nonce: nonce_str.clone(),
            message: message.clone(),
            timestamp,
        },
    );

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
            Json(
                serde_json::json!({"error": "No challenge found. Please request a new challenge."}),
            ),
        )
    })?;

    // Check if challenge has expired (5 minutes)
    let now = chrono::Utc::now().timestamp();
    if now - challenge_data.timestamp > 300 {
        warn!("Challenge expired for address: {}", &payload.address[..16]);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(
                serde_json::json!({ "error": "Challenge expired. Please request a new challenge." }),
            ),
        ));
    }

    info!(
        "Verifying signature for address: {}",
        &payload.address[..16]
    );

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
            info!(
                "✅ Signature verification PASSED for: {}",
                &payload.address[..16]
            );
        }
        Ok(false) => {
            warn!(
                "❌ Signature verification FAILED for: {}",
                &payload.address[..16]
            );
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

    let token = state
        .jwt_manager
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
    address: &str,
    message: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // Decode signature from hex (CIP-30 returns COSE_Sign1)
    let signature_bytes =
        hex::decode(signature_hex).map_err(|e| format!("Invalid signature hex: {}", e))?;

    // Decode public key from hex
    let public_key_bytes =
        hex::decode(public_key_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;

    // Parse COSE_Sign1 structure (CIP-30 format)
    // CIP-30 wallets return signature in COSE_Sign1 format
    // We need to extract the raw signature bytes and payload
    let (raw_signature, payload) = extract_signature_from_cose_sign1(&signature_bytes)
        .map_err(|e| format!("Failed to parse COSE_Sign1: {}", e))?;

    // Verify payload matches the challenge message (if payload is present)
    if !payload.is_empty() {
        let payload_str = String::from_utf8(payload)
            .map_err(|_| "Payload is not valid UTF-8".to_string())?;
        if payload_str != message {
            return Err(
                "Payload in COSE_Sign1 does not match the challenge message".to_string()
            );
        }
    }

    // Parse COSE_Key structure (CIP-30 format)
    // Wallet extensions return public key in COSE_Key format
    // We need to extract the raw public key bytes
    let raw_public_key = extract_public_key_from_cose(&public_key_bytes)
        .map_err(|e| format!("Failed to parse COSE key: {}", e))?;

    // CRITICAL SECURITY CHECK: Verify the public key matches the claimed address
    // This prevents attackers from authenticating as any address with their own keys
    verify_address_from_public_key(address, &raw_public_key)
        .map_err(|e| format!("Address verification failed: {}", e))?;

    // Create Ed25519 verifying key
    let verifying_key = VerifyingKey::from_bytes(&raw_public_key)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    // Parse signature
    let signature = Signature::from_bytes(&raw_signature);

    // Verify signature against the original message
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// Extract raw Ed25519 public key from COSE_Key format
fn extract_public_key_from_cose(cose_key_bytes: &[u8]) -> Result<[u8; 32], String> {
    use ciborium::Value;
    use std::io::Cursor;

    // CIP-30 wallets return COSE_Key in CBOR format (RFC 8152)
    // Structure: CBOR Map with:
    //   kty (1): Key type (1 for OKP)
    //   crv (-1): Curve (6 for Ed25519)
    //   x (-2): Public key bytes (32 bytes)

    // Handle case where bytes are already raw 32-byte key
    if cose_key_bytes.len() == 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(cose_key_bytes);
        return Ok(key);
    }

    // Parse CBOR structure
    let cursor = Cursor::new(cose_key_bytes);
    let value: Value = ciborium::from_reader(cursor)
        .map_err(|e| format!("Failed to parse CBOR: {}", e))?;

    // Extract map from CBOR value
    let map = match value {
        Value::Map(m) => m,
        _ => return Err("COSE_Key must be a CBOR map".to_string()),
    };

    // Look for key -2 (x coordinate / public key)
    for (key, val) in map {
        // Check if key is integer -2
        if let Value::Integer(k) = key {
            if k == ciborium::value::Integer::from(-2) {
                // Extract bytes from value
                if let Value::Bytes(bytes) = val {
                    if bytes.len() == 32 {
                        let mut key_bytes = [0u8; 32];
                        key_bytes.copy_from_slice(&bytes);
                        return Ok(key_bytes);
                    } else {
                        return Err(format!(
                            "Public key must be 32 bytes, got {}",
                            bytes.len()
                        ));
                    }
                } else {
                    return Err("Public key value must be bytes".to_string());
                }
            }
        }
    }

    Err("Could not find public key (label -2) in COSE_Key structure".to_string())
}

// Extract signature and payload from COSE_Sign1 format (CIP-30)
fn extract_signature_from_cose_sign1(cose_sign1_bytes: &[u8]) -> Result<([u8; 64], Vec<u8>), String> {
    use ciborium::Value;
    use std::io::Cursor;

    // COSE_Sign1 structure (RFC 8152):
    // [
    //   protected_headers (bstr),
    //   unprotected_headers (map),
    //   payload (bstr / nil),
    //   signature (bstr)
    // ]

    // Handle case where bytes are already raw 64-byte signature
    if cose_sign1_bytes.len() == 64 {
        let mut sig = [0u8; 64];
        sig.copy_from_slice(cose_sign1_bytes);
        return Ok((sig, Vec::new()));
    }

    // Parse CBOR structure
    let cursor = Cursor::new(cose_sign1_bytes);
    let value: Value = ciborium::from_reader(cursor)
        .map_err(|e| format!("Failed to parse COSE_Sign1 CBOR: {}", e))?;

    // Extract array from CBOR value
    let array = match value {
        Value::Array(arr) => arr,
        _ => return Err("COSE_Sign1 must be a CBOR array".to_string()),
    };

    // Verify array has 4 elements
    if array.len() != 4 {
        return Err(format!(
            "COSE_Sign1 must have 4 elements, got {}",
            array.len()
        ));
    }

    // Extract payload (index 2)
    let payload = match &array[2] {
        Value::Bytes(bytes) => bytes.clone(),
        Value::Null => Vec::new(),
        _ => return Err("COSE_Sign1 payload must be bytes or null".to_string()),
    };

    // Extract signature (index 3)
    let signature_bytes = match &array[3] {
        Value::Bytes(bytes) => bytes.clone(),
        _ => return Err("COSE_Sign1 signature must be bytes".to_string()),
    };

    // Verify signature is 64 bytes (Ed25519)
    if signature_bytes.len() != 64 {
        return Err(format!(
            "Ed25519 signature must be 64 bytes, got {}",
            signature_bytes.len()
        ));
    }

    let mut signature = [0u8; 64];
    signature.copy_from_slice(&signature_bytes);

    Ok((signature, payload))
}

// ============================================================================
// ADDITIONAL: Verify address matches public key
// ============================================================================

fn verify_address_from_public_key(
    address_bech32: &str,
    public_key_bytes: &[u8; 32],
) -> Result<bool, String> {
    use cardano_serialization_lib::{
        address::{Address, BaseAddress, EnterpriseAddress, PointerAddress},
        crypto::PublicKey,
    };

    // Parse the Cardano address from bech32 format
    let address = Address::from_bech32(address_bech32)
        .map_err(|e| format!("Invalid Cardano address: {}", e))?;

    // Create PublicKey from bytes
    let public_key = PublicKey::from_bytes(public_key_bytes)
        .map_err(|e| format!("Invalid public key bytes: {}", e))?;

    // Hash the public key to get the key hash (Blake2b-224)
    let pub_key_hash = public_key.hash();

    // Extract payment credential from address and compare
    // Try different address types (Base, Enterprise, Pointer, etc.)
    let matches = if let Some(base_addr) = BaseAddress::from_address(&address) {
        // Base address (payment + stake)
        match base_addr.payment_cred().to_keyhash() {
            Some(addr_key_hash) => addr_key_hash.to_bytes() == pub_key_hash.to_bytes(),
            None => {
                return Err(
                    "Address uses script credential, not key credential".to_string()
                )
            }
        }
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&address) {
        // Enterprise address (payment only, no stake)
        match enterprise_addr.payment_cred().to_keyhash() {
            Some(addr_key_hash) => addr_key_hash.to_bytes() == pub_key_hash.to_bytes(),
            None => {
                return Err(
                    "Address uses script credential, not key credential".to_string()
                )
            }
        }
    } else if let Some(pointer_addr) = PointerAddress::from_address(&address) {
        // Pointer address
        match pointer_addr.payment_cred().to_keyhash() {
            Some(addr_key_hash) => addr_key_hash.to_bytes() == pub_key_hash.to_bytes(),
            None => {
                return Err(
                    "Address uses script credential, not key credential".to_string()
                )
            }
        }
    } else {
        return Err("Unsupported address type (Byron, Reward, or Script)".to_string());
    };

    if matches {
        Ok(true)
    } else {
        Err("Public key does not match the address".to_string())
    }
}
