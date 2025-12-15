use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use super::jwt::JwtManager;

pub async fn auth_middleware(
    // Arc is a smart pointer that allows multiple threads to share ownership of the same value safely
    // In this case JwtManager
    // Whereas State (Axum Extractor) is the way in which this function can get access to the value of the JwtManager
    // through a variable called jwt_manager
    // Main differenc between State and Arc is that when state is used it creates a new instance so the previous one and the newly created are totally different
    // And for the Arc it doesnot create new instance rather creates another reference to the same data
    State(jwt_manager): State<Arc<JwtManager>>,
    // Used for checking the Authorization Header for checking the token
    headers: HeaderMap,
    // It will recieve what request has been recieved i.e Http methods, url and body
    mut request: Request,
    next: Next
    ) -> Result<Response, (StatusCode, Json<serde_json::Value>)>{


        // taking the token out of the header
        let token = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Missing authorization token"})),
                )
            })?;


        // Take the claim data if the token is valid
        let claims = jwt_manager.validate_token(token).map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": format!("Invalid token: {}", e) })),
                )
        })?;

        request.extensions_mut().insert(claims);
        Ok(next.run(request).await)
    }
