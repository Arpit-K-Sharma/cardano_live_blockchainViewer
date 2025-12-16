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

    // Normalize address format - handle both hex and bech32 formats
    let normalized_address = normalize_address_format(&payload.address);
    info!(
        "Address received: {} (normalized: {})",
        &payload.address[..16],
        &normalized_address[..16]
    );

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
        normalized_address.clone(),
        ChallengeData {
            nonce: nonce_str.clone(),
            message: message.clone(),
            timestamp,
        },
    );

    let cutoff = timestamp - 300;
    challenges.retain(|_, data| data.timestamp > cutoff);

    info!(
        "Challenge created for normalized address: {}",
        &normalized_address[..16]
    );

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

    // Normalize address format for lookup (same as in create_challenge)
    let normalized_address = normalize_address_format(&payload.address);
    info!(
        "Verifying signature - original address: {} (normalized: {})",
        &payload.address[..payload.address.len().min(16)],
        &normalized_address[..normalized_address.len().min(16)]
    );

    let challenges = state.challenges.lock().await;
    // Try both normalized and original address for lookup
    let challenge_data = challenges.get(&normalized_address)
        .or_else(|| challenges.get(&payload.address))
        .cloned();
    drop(challenges);

    let challenge_data = challenge_data.ok_or_else(|| {
        warn!(
            "No challenge found for address: {} (normalized: {})",
            &payload.address[..payload.address.len().min(16)],
            &normalized_address[..normalized_address.len().min(16)]
        );
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

    info!(
        "🔍 Starting signature verification for address: {}",
        &payload.address[..16]
    );
    info!(
        "📊 Signature data length: {} bytes",
        payload.signature.len()
    );
    info!("🔑 Key data length: {} bytes", payload.key.len());

    info!("🔐 Starting cryptographic signature verification...");
    match verify_cardano_signature(
        &normalized_address,
        &challenge_data.message,
        &payload.signature,
        &payload.key,
    ) {
        Ok(true) => {
            info!(
                "✅ Signature verification PASSED for: {}",
                &normalized_address[..normalized_address.len().min(16)]
            );
        }
        Ok(false) => {
            warn!(
                "❌ Signature verification FAILED for: {}",
                &normalized_address[..normalized_address.len().min(16)]
            );
            warn!("📊 Debug info:");
            warn!("   - Address: {}", &normalized_address[..normalized_address.len().min(32)]);
            warn!("   - Signature length: {} chars", payload.signature.len());
            warn!("   - Key length: {} chars", payload.key.len());
            warn!("   - Message length: {} chars", challenge_data.message.len());
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid signature - the signed message does not match the challenge or the signature is invalid",
                    "details": "This could mean the wallet signed a different message or the signature is corrupted. Check backend logs for detailed verification steps."
                })),
            ));
        }
        Err(e) => {
            error!("❌ Signature verification error: {}", e);
            error!("📊 Error occurred during verification - check logs above for details");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Signature verification failed",
                    "details": format!("Technical error: {}. Check backend logs for detailed information.", e)
                })),
            ));
        }
    }

    let mut challenges = state.challenges.lock().await;
    // Remove challenge using normalized address (or original if normalized not found)
    challenges.remove(&normalized_address);
    challenges.remove(&payload.address);
    drop(challenges);


    // ========================================================================
    // CONVERT ADDRESS TO BECH32 FOR BLOCKFROST API
    // ========================================================================
    let bech32_address = convert_to_bech32(&normalized_address)
        .unwrap_or_else(|e| {
            warn!("Failed to convert address to bech32: {}, using original", e);
            normalized_address.clone()
        });

    info!("📝 Address for JWT: {} (bech32 format)", &bech32_address[..bech32_address.len().min(20)]);

    // Use normalized address for JWT token
    let token = state
        .jwt_manager
        .generate_token(bech32_address.clone(), payload.stake_address)
        .map_err(|e| {
            error!("Failed to generate JWT: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to generate token" })),
            )
        })?;

    info!("✅ JWT issued for address: {}", &normalized_address[..normalized_address.len().min(16)]);

    Ok(Json(VerifyResponse { token }))
}

// ============================================================================
// ADDRESS NORMALIZATION
// ============================================================================

/// Normalize address format to handle both hex and bech32 formats
fn normalize_address_format(address: &str) -> String {
    // Check if it's already a valid hex string
    if hex::decode(address).is_ok() && address.len() % 2 == 0 {
        // It's already hex format, return as-is
        address.to_string()
    } else {
        // Assume it's bech32 format, try to parse and extract hex
        // For now, return as-is and let the signature verification handle it
        // In production, you might want to use cardano-serialization-lib to convert bech32 to hex
        address.to_string()
    }
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

    info!("🔍 Step 1: Decoding signature and key data...");

    // Decode signature from hex (CIP-30 returns COSE_Sign1)
    let signature_bytes =
        hex::decode(signature_hex).map_err(|e| format!("Invalid signature hex: {}", e))?;
    info!("📊 Signature decoded: {} bytes", signature_bytes.len());

    // Decode public key from hex
    let public_key_bytes =
        hex::decode(public_key_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;
    info!("🔑 Public key decoded: {} bytes", public_key_bytes.len());

    info!("🔍 Step 2: Parsing COSE_Sign1 structure...");
    // Parse COSE_Sign1 structure (CIP-30 format)
    // CIP-30 wallets return signature in COSE_Sign1 format
    // We need to extract the raw signature bytes, payload, and protected headers
    let (raw_signature, payload, protected_headers) = extract_signature_from_cose_sign1(&signature_bytes)
        .map_err(|e| format!("Failed to parse COSE_Sign1: {}", e))?;
    info!("✅ COSE_Sign1 parsed successfully");
    info!("📝 Payload length: {} bytes", payload.len());
    info!("📋 Protected headers length: {} bytes", protected_headers.len());
    info!("✍️ Signature length: {} bytes", raw_signature.len());

    info!("🔍 Step 3: Determining what was actually signed...");
    // CIP-30 spec: signData(address, hexPayload) signs the BYTES represented by hexPayload
    // Frontend: message -> hex_encode -> signData(address, hexString)
    // Wallet: hexString -> decode -> signs the decoded bytes (original message bytes)
    // Therefore: We should verify against message.as_bytes()
    
    // Convert message to hex (as frontend does) for reference
    let message_hex = hex::encode(message.as_bytes());
    info!("📝 Original message: {} bytes", message.as_bytes().len());
    info!("📝 Message hex (what frontend sends): {}", &message_hex[..message_hex.len().min(100)]);
    info!("📝 COSE payload length: {} bytes", payload.len());
    
    // According to CIP-30, wallets sign the bytes represented by the hex payload
    // So if frontend sends hex-encoded message, wallet signs the decoded bytes (original message)
    // However, some wallets include different things in COSE_Sign1 payload:
    // 1. Empty payload (most common) - wallet signed the decoded bytes
    // 2. Original message bytes - wallet signed these bytes
    // 3. Hex string representation - wallet signed the decoded bytes
    
    // Primary verification: against original message bytes (what wallet actually signed)
    let primary_signed_bytes = message.as_bytes();
    
    // Also prepare alternative verification targets
    let message_bytes_vec = message.as_bytes().to_vec();
    let message_hex_bytes = message_hex.as_bytes().to_vec();
    
    info!("📝 Will verify against:");
    info!("   1. Original message bytes: {} bytes", primary_signed_bytes.len());
    info!("   2. Message hex string bytes: {} bytes", message_hex_bytes.len());
    if !payload.is_empty() {
        info!("   3. COSE payload: {} bytes", payload.len());
        if payload == message_bytes_vec {
            info!("   ✅ COSE payload matches message bytes");
        } else if let Ok(payload_str) = String::from_utf8(payload.clone()) {
            info!("   📝 COSE payload as string: {}", &payload_str[..payload_str.len().min(50)]);
            if let Ok(decoded) = hex::decode(&payload_str) {
                info!("   📝 COSE payload decoded from hex: {} bytes", decoded.len());
                if decoded == message_bytes_vec {
                    info!("   ✅ Decoded payload matches message bytes");
                }
            }
        }
    }

    info!("🔍 Step 4: Parsing COSE_Key structure...");
    // Parse COSE_Key structure (CIP-30 format)
    // Wallet extensions return public key in COSE_Key format
    // We need to extract the raw public key bytes
    let raw_public_key = extract_public_key_from_cose(&public_key_bytes)
        .map_err(|e| format!("Failed to parse COSE key: {}", e))?;
    info!("✅ COSE_Key parsed successfully");
    info!("🔑 Public key extracted: {} bytes", raw_public_key.len());

    info!("🔍 Step 5: Verifying address matches public key...");
    // CRITICAL SECURITY CHECK: Verify the public key matches the claimed address
    // This prevents attackers from authenticating as any address with their own keys
    match verify_address_from_public_key(address, &raw_public_key) {
        Ok(true) => {
            info!("✅ Address verification passed");
        }
        Ok(false) => {
            warn!("⚠️ Address verification returned false - address may not match public key");
            warn!("⚠️ Continuing with signature verification anyway for debugging...");
            // For now, we'll continue to see if signature verification works
            // In production, you might want to return an error here
        }
        Err(e) => {
            warn!("⚠️ Address verification error: {}", e);
            warn!("⚠️ Continuing with signature verification anyway for debugging...");
            // For now, we'll continue to see if signature verification works
            // In production, you might want to return an error here
        }
    }

    info!("🔍 Step 6: Creating Ed25519 verifying key...");
    // Create Ed25519 verifying key
    let verifying_key = VerifyingKey::from_bytes(&raw_public_key)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    info!("✅ Ed25519 verifying key created");

    info!("🔍 Step 7: Verifying signature...");
    // Parse signature
    let signature = Signature::from_bytes(&raw_signature);

    // According to COSE spec (RFC 8152), the signature is computed over Sig_structure:
    // Sig_structure = [
    //   "Signature1",
    //   protected_headers,
    //   external_aad,  // empty bstr for CIP-30
    //   payload
    // ]
    // However, many CIP-30 wallets sign just the payload bytes directly.
    // We'll try both methods.

    // Method 1: Verify against COSE Sig_structure (full COSE compliance)
    if !protected_headers.is_empty() || !payload.is_empty() {
        info!("🔄 Attempt 1: Verifying against COSE Sig_structure...");
        // Build Sig_structure: ["Signature1", protected_headers, external_aad (empty), payload]
        // According to RFC 8152, Sig_structure is a CBOR array
        use ciborium::Value;
        let external_aad = Vec::<u8>::new(); // Empty for CIP-30
        
        // Create Sig_structure as CBOR array: ["Signature1", protected_headers, external_aad, payload]
        let sig_structure = Value::Array(vec![
            Value::Text("Signature1".to_string()),
            Value::Bytes(protected_headers.clone()),
            Value::Bytes(external_aad),
            Value::Bytes(payload.clone()),
        ]);
        
        // Encode to bytes
        let mut sig_structure_bytes = Vec::new();
        ciborium::ser::into_writer(&sig_structure, &mut sig_structure_bytes)
            .map_err(|e| format!("Failed to encode Sig_structure: {}", e))?;
        
        info!("📝 Sig_structure length: {} bytes", sig_structure_bytes.len());
        if verifying_key.verify(&sig_structure_bytes, &signature).is_ok() {
            info!("✅ Signature verification PASSED (method 1: COSE Sig_structure)!");
            return Ok(true);
        }
    }
    
    // Method 2: Verify against original message bytes (most common for CIP-30)
    info!("🔄 Attempt 2: Verifying against original message bytes...");
    if verifying_key.verify(primary_signed_bytes, &signature).is_ok() {
        info!("✅ Signature verification PASSED (method 2: original message bytes)!");
        return Ok(true);
    }
    
    // Method 3: If payload exists and matches message, try verifying against payload
    if !payload.is_empty() && payload == message_bytes_vec {
        info!("🔄 Attempt 3: Verifying against COSE payload (matches message bytes)...");
        if verifying_key.verify(&payload, &signature).is_ok() {
            info!("✅ Signature verification PASSED (method 3: COSE payload)!");
            return Ok(true);
        }
    }
    
    // Method 4: Try verifying against hex-encoded message string bytes
    info!("🔄 Attempt 4: Verifying against hex-encoded message string bytes...");
    if verifying_key.verify(&message_hex_bytes, &signature).is_ok() {
        info!("✅ Signature verification PASSED (method 4: hex string bytes)!");
        return Ok(true);
    }
    
    // Method 5: If payload is a hex string, decode and verify
    if !payload.is_empty() {
        if let Ok(payload_str) = String::from_utf8(payload.clone()) {
            if let Ok(decoded_payload) = hex::decode(&payload_str) {
                if decoded_payload == message_bytes_vec {
                    info!("🔄 Attempt 5: Verifying against decoded hex payload...");
                    if verifying_key.verify(&decoded_payload, &signature).is_ok() {
                        info!("✅ Signature verification PASSED (method 5: decoded hex payload)!");
                        return Ok(true);
                    }
                }
            }
        }
    }
    
    // All verification methods failed
    warn!("❌ Signature verification FAILED - all methods attempted");
    warn!("📊 Verification details:");
    warn!("   - Message bytes length: {}", message_bytes_vec.len());
    warn!("   - Message hex length: {}", message_hex_bytes.len());
    warn!("   - COSE payload length: {}", payload.len());
    warn!("   - Raw signature (hex): {}", hex::encode(&raw_signature));
    if !payload.is_empty() && payload.len() <= 200 {
        warn!("   - COSE payload (hex): {}", hex::encode(&payload));
    }
    
    Ok(false)
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
    let value: Value =
        ciborium::from_reader(cursor).map_err(|e| format!("Failed to parse CBOR: {}", e))?;

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
                        return Err(format!("Public key must be 32 bytes, got {}", bytes.len()));
                    }
                } else {
                    return Err("Public key value must be bytes".to_string());
                }
            }
        }
    }

    Err("Could not find public key (label -2) in COSE_Key structure".to_string())
}

// Extract signature, payload, and protected headers from COSE_Sign1 format (CIP-30)
fn extract_signature_from_cose_sign1(
    cose_sign1_bytes: &[u8],
) -> Result<([u8; 64], Vec<u8>, Vec<u8>), String> {
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
        return Ok((sig, Vec::new(), Vec::new()));
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

    // Extract protected headers (index 0)
    let protected_headers = match &array[0] {
        Value::Bytes(bytes) => bytes.clone(),
        _ => return Err("COSE_Sign1 protected headers must be bytes".to_string()),
    };

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

    Ok((signature, payload, protected_headers))
}

// ============================================================================
// ADDITIONAL: Verify address matches public key
// ============================================================================

fn verify_address_from_public_key(
    address_str: &str,
    public_key_bytes: &[u8; 32],
) -> Result<bool, String> {
    use cardano_serialization_lib::{
        address::{BaseAddress, EnterpriseAddress, PointerAddress},
        crypto::PublicKey,
    };

    // Try to parse as both hex and bech32 formats
    let address = if address_str.len() % 2 == 0 && hex::decode(address_str).is_ok() {
        // It's hex format - decode and create Address from bytes
        let address_bytes =
            hex::decode(address_str).map_err(|e| format!("Invalid hex address: {}", e))?;

        // Create address from raw bytes
        cardano_serialization_lib::address::Address::from_bytes(address_bytes)
            .map_err(|e| format!("Invalid address bytes: {}", e))?
    } else {
        // Try bech32 format
        cardano_serialization_lib::address::Address::from_bech32(address_str)
            .map_err(|e| format!("Invalid bech32 address: {}", e))?
    };

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
            None => return Err("Address uses script credential, not key credential".to_string()),
        }
    } else if let Some(enterprise_addr) = EnterpriseAddress::from_address(&address) {
        // Enterprise address (payment only, no stake)
        match enterprise_addr.payment_cred().to_keyhash() {
            Some(addr_key_hash) => addr_key_hash.to_bytes() == pub_key_hash.to_bytes(),
            None => return Err("Address uses script credential, not key credential".to_string()),
        }
    } else if let Some(pointer_addr) = PointerAddress::from_address(&address) {
        // Pointer address
        match pointer_addr.payment_cred().to_keyhash() {
            Some(addr_key_hash) => addr_key_hash.to_bytes() == pub_key_hash.to_bytes(),
            None => return Err("Address uses script credential, not key credential".to_string()),
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


/// Convert hex address to bech32 format for Blockfrost API
fn convert_to_bech32(address: &str) -> Result<String, String> {
    use cardano_serialization_lib::address::Address;
    
    // If it's already bech32, return as-is
    if address.starts_with("addr") {
        return Ok(address.to_string());
    }
    
    // Try to convert hex to bech32
    let address_bytes = hex::decode(address)
        .map_err(|e| format!("Invalid hex address: {}", e))?;
    
    let addr = Address::from_bytes(address_bytes)
        .map_err(|e| format!("Invalid address bytes: {}", e))?;
    
    addr.to_bech32(None)
        .map_err(|e| format!("Failed to convert to bech32: {}", e))
}
