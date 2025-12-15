use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub wallet_address: String,
    pub stake_address: Option<String>,
    pub exp: usize,
    // issued at
    pub iat: usize,
}

pub struct JwtManager {
    secret: String,
}

impl JwtManager {

    // It creates an instance for the JwtManager everytime it is called
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    // A function inside a impl which can be also known as class in rust, which takes an instance, wallet address and stake address
    // Here the type Return<String, String> means that the first String is the json web token and the second is the error string
    // So you can say Result<String, String> -> Result < Sucess, Error >
    pub fn generate_token(&self, wallet_address: String, stake_address: Option<String>) -> Result<String, String> {
        
        // Token Expires in 24 hours
        let now  = chrono::Utc::now();
        let expiration = now
            .checked_add_signed(chrono::Duration::hours(24))
            .ok_or("Failed to calculate expiration")?
            .timestamp() as usize;

        let claims = Claims {
            wallet_address,
            stake_address,
            exp: expiration,
            iat: now.timestamp() as usize,
        };

        // Create a JWT string by combining a header, payload, and a secret key.
        encode(
            // sets the default header (algorithm = HS256)
            &Header::default(),
            // the payload
            &claims,
            // The secret key used to sign the token
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| format!("Failed to encode JWT: {}", e))
    }

    // For validating the token
    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default()
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {}", e))
    }
}