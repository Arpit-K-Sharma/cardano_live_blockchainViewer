// src/api/user.rs
use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::Claims;
use crate::blockfrost::BlockfrostClient;

#[derive(Clone)]
pub struct UserState {
    pub blockfrost: Arc<BlockfrostClient>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub address: String,
    pub page: Option<u32>,
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub transactions: Vec<Transaction>,
    pub total: usize,
    pub page: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Transaction {
    pub tx_hash: String,
    pub block: String,
    pub block_height: u64,
    pub block_time: u64,
    pub slot: u64,
    pub index: u32,
    pub fees: String,
}

#[derive(Debug, Serialize)]
pub struct WalletSummary {
    pub address: String,
    pub stake_address: Option<String>,
    pub balance: String,
    pub transaction_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub balance: String,
    pub tx_count: usize,
}

pub async fn get_transactions(
    State(state): State<UserState>,
    Extension(_claims): Extension<Claims>, // JWT still required for authentication
    axum::extract::Query(query): axum::extract::Query<TransactionQuery>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validate wallet address from query parameter
    if query.address.is_empty() {
        tracing::error!("Empty wallet address in query parameter");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing wallet address parameter" })),
        ));
    }

    let page = query.page.unwrap_or(1);
    let count = query.count.unwrap_or(10);

    let address_preview = if query.address.len() >= 16 {
        &query.address[..16]
    } else {
        &query.address
    };

    tracing::info!(
        "Fetching transactions for address: {}... (page: {})",
        address_preview,
        page
    );

    let transactions = state
        .blockfrost
        .get_address_transactions(&query.address, page, count)
        .await
        .map_err(|e| {
            tracing::error!("Blockfrost error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to fetch transactions: {}", e) })),
            )
        })?;

    Ok(Json(TransactionResponse {
        total: transactions.len(),
        page,
        transactions,
    }))
}

pub async fn get_summary(
    State(state): State<UserState>,
    Extension(claims): Extension<Claims>, // JWT still required for authentication and stake address
    axum::extract::Query(query): axum::extract::Query<SummaryQuery>,
) -> Result<Json<WalletSummary>, (StatusCode, Json<serde_json::Value>)> {
    // Validate wallet address from query parameter
    if query.address.is_empty() {
        tracing::error!("Empty wallet address in query parameter");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing wallet address parameter" })),
        ));
    }

    let address_preview = if query.address.len() >= 16 {
        &query.address[..16]
    } else {
        &query.address
    };

    tracing::info!(
        "Fetching wallet summary for address: {}...",
        address_preview
    );

    let account_info = state
        .blockfrost
        .get_account_info(&query.address)
        .await
        .map_err(|e| {
            tracing::error!("Blockfrost error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to fetch account info: {}", e) })),
            )
        })?;

    Ok(Json(WalletSummary {
        address: query.address,
        stake_address: claims.stake_address, // Still get stake address from JWT
        balance: account_info.balance,
        transaction_count: account_info.tx_count,
    }))
}