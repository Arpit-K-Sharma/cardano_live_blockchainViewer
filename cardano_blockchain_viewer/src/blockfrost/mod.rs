use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Convert hex address to bech32 format for Blockfrost API
/// Blockfrost requires bech32 addresses (addr1...), not hex
fn hex_to_bech32_address(hex_address: &str) -> Result<String, String> {
    use cardano_serialization_lib::address::Address;
    
    // Try to decode hex address
    let address_bytes = hex::decode(hex_address)
        .map_err(|e| format!("Invalid hex address: {}", e))?;
    
    // Create Address from bytes
    let address = Address::from_bytes(address_bytes)
        .map_err(|e| format!("Invalid address bytes: {}", e))?;
    
    // Convert to bech32
    address.to_bech32(None)
        .map_err(|e| format!("Failed to convert to bech32: {}", e))
}

/// Normalize address format - convert hex to bech32 if needed
/// Returns bech32 address if input is hex, otherwise returns as-is
fn normalize_address_for_blockfrost(address: &str) -> Result<String, String> {
    // Check if it's already bech32 (starts with addr)
    if address.starts_with("addr") {
        return Ok(address.to_string());
    }
    
    // Try to convert hex to bech32
    hex_to_bech32_address(address)
}

#[derive(Clone)]
pub struct BlockfrostClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockfrostTransaction {
    pub tx_hash: String,
    pub block: String,
    pub block_height: u64,
    pub block_time: u64,
    pub slot: u64,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockfrostTxDetails {
    pub fees: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockfrostAddressInfo {
    pub address: String,
    #[serde(default)]
    pub amount: Vec<BlockfrostAmount>,
    #[serde(default)]
    pub stake_address: Option<String>,
    #[serde(default)]
    pub tx_count: usize,
    // Blockfrost returns "type" field
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockfrostAmount {
    pub unit: String,
    pub quantity: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct AccountInfo {
//     pub controlled_amount: String,
//     pub stake_address: Option<String>,
//     pub tx_count: usize,
// }

impl BlockfrostClient {
    pub fn new(api_key: String, network: &str) -> Self {
        let base_url = match network {
            "mainnet" => "https://cardano-mainnet.blockfrost.io/api/v0",
            "preprod" => "https://cardano-preprod.blockfrost.io/api/v0",
            "preview" => "https://cardano-preview.blockfrost.io/api/v0",
            _ => "https://cardano-preprod.blockfrost.io/api/v0",
        };

        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.to_string(),
        }
    }

    pub async fn get_address_transactions(
        &self,
        address: &str,
        page: u32,
        count: u32,
    ) -> Result<Vec<crate::api::user::Transaction>, String> {
        // Convert hex address to bech32 if needed (Blockfrost requires bech32)
        let bech32_address = normalize_address_for_blockfrost(address)
            .map_err(|e| format!("Address conversion failed: {}", e))?;
        
        tracing::info!(
            "Blockfrost: Converting address {} -> {}",
            &address[..address.len().min(16)],
            &bech32_address[..bech32_address.len().min(20)]
        );
        
        let url = format!("{}/addresses/{}/transactions", self.base_url, bech32_address);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .query(&[("page", page.to_string()), ("count", count.to_string())])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Blockfrost API error: {} - {}", status, error_text);
            return Err(format!("Blockfrost error: {} - {}", status, error_text));
        }

        let txs: Vec<BlockfrostTransaction> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut transactions = Vec::new();
        for tx in txs {
            let details = self.get_transaction_details(&tx.tx_hash).await?;
            transactions.push(crate::api::user::Transaction {
                tx_hash: tx.tx_hash,
                block: tx.block,
                block_height: tx.block_height,
                block_time: tx.block_time,
                slot: tx.slot,
                index: tx.index,
                fees: details.fees,
            });
        }

        Ok(transactions)
    }

    async fn get_transaction_details(&self, tx_hash: &str) -> Result<BlockfrostTxDetails, String> {
        let url = format!("{}/txs/{}", self.base_url, tx_hash);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Blockfrost API error: {} - {}", status, error_text);
            return Err(format!("Blockfrost error: {} - {}", status, error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_account_info(&self, address: &str) -> Result<crate::api::user::AccountInfo, String> {
        // Convert hex address to bech32 if needed (Blockfrost requires bech32)
        let bech32_address = normalize_address_for_blockfrost(address)
            .map_err(|e| format!("Address conversion failed: {}", e))?;
        
        tracing::info!(
            "Blockfrost: Converting address {} -> {}",
            &address[..address.len().min(16)],
            &bech32_address[..bech32_address.len().min(20)]
        );
        
        let url = format!("{}/addresses/{}", self.base_url, bech32_address);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Blockfrost API error: {} - {}", status, error_text);
            return Err(format!("Blockfrost error: {} - {}", status, error_text));
        }

        let info: BlockfrostAddressInfo = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse Blockfrost response: {}", e);
                format!("Failed to parse response: {}", e)
            })?;

        // Extract ADA balance (unit = "lovelace")
        let balance = info.amount
            .iter()
            .find(|a| a.unit == "lovelace")
            .map(|a| a.quantity.clone())
            .unwrap_or_else(|| "0".to_string());

        Ok(crate::api::user::AccountInfo {
            balance,
            tx_count: info.tx_count,
        })
    }
}