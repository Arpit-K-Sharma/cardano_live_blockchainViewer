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
    pub page: Option<u32>,
    pub count: Option<u32>,
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
    Extension(claims): Extension<Claims>,
    axum::extract::Query(query): axum::extract::Query<TransactionQuery>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let page = query.page.unwrap_or(1);
    let count = query.count.unwrap_or(10);

    tracing::info!(
        "Fetching transactions for address: {} (page: {})",
        &claims.wallet_address[..16],
        page
    );

    let transactions = state
        .blockfrost
        .get_address_transactions(&claims.wallet_address, page, count)
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
    Extension(claims): Extension<Claims>,
) -> Result<Json<WalletSummary>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        "Fetching wallet summary for address: {}",
        &claims.wallet_address[..16]
    );

    let account_info = state
        .blockfrost
        .get_account_info(&claims.wallet_address)
        .await
        .map_err(|e| {
            tracing::error!("Blockfrost error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to fetch account info: {}", e) })),
            )
        })?;

    Ok(Json(WalletSummary {
        address: claims.wallet_address,
        stake_address: claims.stake_address,
        balance: account_info.balance,
        transaction_count: account_info.tx_count,
    }))
}