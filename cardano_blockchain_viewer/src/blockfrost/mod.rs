use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Convert hex address to bech32 format for Blockfrost API
/// Blockfrost requires bech32 addresses (addr1...), not hex
fn hex_to_bech32_address(hex_address: &str) -> Result<String, String> {
    use cardano_serialization_lib::address::Address;
    
    tracing::debug!("Converting hex address to bech32: {} ({} chars)", &hex_address[..hex_address.len().min(32)], hex_address.len());
    
    // Try to decode hex address
    let address_bytes = hex::decode(hex_address)
        .map_err(|e| {
            tracing::error!("Failed to decode hex address: {}", e);
            format!("Invalid hex address: {}", e)
        })?;
    
    tracing::debug!("Decoded {} bytes from hex", address_bytes.len());
    
    // Create Address from bytes
    let address = Address::from_bytes(address_bytes)
        .map_err(|e| {
            tracing::error!("Failed to create Address from bytes: {}", e);
            format!("Invalid address bytes: {}", e)
        })?;
    
    // Convert to bech32
    let bech32 = address.to_bech32(None)
        .map_err(|e| {
            tracing::error!("Failed to convert to bech32: {}", e);
            format!("Failed to convert to bech32: {}", e)
        })?;
    
    tracing::debug!("Converted to bech32: {} ({} chars)", &bech32[..bech32.len().min(32)], bech32.len());
    
    Ok(bech32)
}

/// Detect network from address format
/// Returns "mainnet", "testnet", or "unknown"
fn detect_network_from_address(address: &str) -> &'static str {
    if address.starts_with("addr1") {
        "mainnet"
    } else if address.starts_with("addr_test") {
        "testnet"
    } else {
        "unknown"
    }
}

/// Normalize address format - convert hex to bech32 if needed
/// Returns bech32 address if input is hex, otherwise returns as-is
fn normalize_address_for_blockfrost(address: &str) -> Result<String, String> {
    tracing::debug!("Normalizing address: {} ({} chars)", &address[..address.len().min(32)], address.len());
    
    // Check if it's already bech32 (starts with addr)
    if address.starts_with("addr") {
        tracing::debug!("Address is already bech32 format");
        let network = detect_network_from_address(address);
        tracing::info!("Detected address network: {} (address: {}...)", network, &address[..address.len().min(20)]);
        return Ok(address.to_string());
    }
    
    // Check if it looks like hex (even length, hex characters)
    if address.len() % 2 == 0 && address.chars().all(|c| c.is_ascii_hexdigit()) {
        tracing::debug!("Address appears to be hex format, converting...");
        return hex_to_bech32_address(address);
    }
    
    // If it doesn't match either format, try hex conversion anyway
    tracing::warn!("Address format unclear, attempting hex conversion: {}", &address[..address.len().min(32)]);
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
    pub tx_index: Option<u32>,
    pub block_height: u64,
    pub block_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockfrostTxDetails {
    pub hash: String,
    pub block: String,
    pub block_height: u64,
    pub block_time: u64,
    pub slot: Option<u64>,
    pub index: Option<u32>,
    pub fees: String,
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
        let raw_base = match network {
            "mainnet" => "https://cardano-mainnet.blockfrost.io/api/v0",
            "preprod" => "https://cardano-preprod.blockfrost.io/api/v0",
            "preview" => "https://cardano-preview.blockfrost.io/api/v0",
            _ => "https://cardano-preprod.blockfrost.io/api/v0",
        };

        // Defensive: ensure /api/v0 is present even if an env override strips it
        // Ensure we have /api/v0 and a trailing slash so Url::join treats it as a path prefix
        let with_v0 = if raw_base.contains("/api/v0") {
            raw_base.to_string()
        } else {
            format!("{}/api/v0", raw_base.trim_end_matches('/'))
        };
        let base_url = if with_v0.ends_with('/') {
            with_v0
        } else {
            format!("{}/", with_v0)
        };
        tracing::info!("Blockfrost base URL: {}", base_url);

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
        
        // Detect network from address and warn if mismatch
        let address_network = detect_network_from_address(&bech32_address);
        let configured_network = if self.base_url.contains("mainnet") {
            "mainnet"
        } else if self.base_url.contains("preprod") {
            "preprod"
        } else if self.base_url.contains("preview") {
            "preview"
        } else {
            "unknown"
        };
        
        if address_network == "mainnet" && configured_network != "mainnet" {
            tracing::warn!(
                "⚠️  Network mismatch detected! Address is mainnet (addr1...) but Blockfrost is configured for {}",
                configured_network
            );
            tracing::warn!("   This will likely result in no data being returned. Consider using a {} address or configuring Blockfrost for mainnet.", configured_network);
        } else if address_network == "testnet" && configured_network == "mainnet" {
            tracing::warn!(
                "⚠️  Network mismatch detected! Address is testnet (addr_test...) but Blockfrost is configured for mainnet"
            );
        }
        
        tracing::info!(
            "Blockfrost: Converting address {} -> {}",
            &address[..address.len().min(16)],
            &bech32_address[..bech32_address.len().min(20)]
        );
        
        // Build URL with proper encoding - Blockfrost requires URL-encoded addresses
        // Use reqwest's URL builder to ensure proper encoding
        let base = reqwest::Url::parse(&self.base_url)
            .map_err(|e| format!("Invalid base URL: {}", e))?;
        
        // Use percent_encoding for URL encoding (standard library approach)
        // reqwest::Url::join() should handle encoding, but we'll be explicit
        let path_segment = format!("addresses/{}/transactions", bech32_address);
        let url = base.join(&path_segment)
            .map_err(|e| format!("Failed to build URL: {}", e))?;
        let url_str = url.as_str();
        
        tracing::info!("Blockfrost: Fetching transactions");
        tracing::info!("  Original address: {} ({} chars)", &address[..address.len().min(32)], address.len());
        tracing::info!("  Bech32 address: {} ({} chars)", &bech32_address[..bech32_address.len().min(32)], bech32_address.len());
        tracing::info!("  URL: {}", url_str);
        tracing::info!("  Page: {}, Count: {}", page, count);

        let response = self
            .client
            .get(url_str)
            .header("project_id", &self.api_key)
            .header("accept", "application/json")
            .query(&[("page", page.to_string()), ("count", count.to_string())])
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Blockfrost request error: {}", e);
                format!("Request failed: {}", e)
            })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            // Special-case: Blockfrost returns 404 when no transactions exist for the address.
            if status.as_u16() == 404 {
                tracing::info!("Blockfrost: No transactions found (404) for {}", &bech32_address[..bech32_address.len().min(20)]);
                return Ok(Vec::new());
            }

            tracing::error!("Blockfrost API error: {} - {}", status, text);
            // Check if response is HTML (error page)
            if text.trim_start().starts_with("<!DOCTYPE") || text.trim_start().starts_with("<html") {
                return Err(format!(
                    "Blockfrost returned HTML instead of JSON (status: {}). This usually means:\n\
                    1. Invalid API key or API key not configured for this network\n\
                    2. Network mismatch between address and Blockfrost configuration\n\
                    3. Malformed request URL\n\
                    Address: {}...\n\
                    URL: {}",
                    status,
                    &bech32_address[..bech32_address.len().min(20)],
                    url_str
                ));
            }
            return Err(format!("Blockfrost error: {} - {}", status, text));
        }

        // Check if response is HTML (shouldn't happen with 200 status, but just in case)
        if text.trim_start().starts_with("<!DOCTYPE") || text.trim_start().starts_with("<html") {
            tracing::error!("Blockfrost returned HTML instead of JSON even with success status");
            return Err(format!(
                "Blockfrost returned HTML instead of JSON. This suggests a configuration issue.\n\
                Address: {}...\n\
                URL: {}",
                &bech32_address[..bech32_address.len().min(20)],
                url_str
            ));
        }

        let preview = if text.len() > 1000 { format!("{}... ({} bytes)", &text[..1000], text.len()) } else { text.clone() };
        let txs: Vec<BlockfrostTransaction> = serde_json::from_str(&text)
            .map_err(|e| {
                tracing::error!("Blockfrost JSON parse error: {}. Body: {}", e, preview);
                format!("Failed to parse response: {}. Body: {}", e, preview)
            })?;

        // If no transactions, return empty list
        if txs.is_empty() {
            tracing::info!("Blockfrost: No transactions found for address");
            return Ok(Vec::new());
        }

        tracing::info!("Blockfrost: Found {} transactions, fetching details...", txs.len());

        // Fetch details with better error handling
        // Limit concurrent requests to avoid rate limiting
        let mut transactions = Vec::new();
        for (idx, tx) in txs.iter().enumerate() {
            // Add a small delay between requests to avoid rate limiting
            if idx > 0 && idx % 5 == 0 {
                tracing::info!("Blockfrost: Processed {}/{} transactions...", idx, txs.len());
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Try to get transaction details, but don't fail if it doesn't work
            match self.get_transaction_details(&tx.tx_hash).await {
                Ok(details) => {
                    transactions.push(crate::api::user::Transaction {
                        tx_hash: tx.tx_hash.clone(),
                        block: details.block,
                        block_height: details.block_height,
                        block_time: details.block_time,
                        slot: details.slot.unwrap_or_default(),
                        index: details.index.unwrap_or_else(|| tx.tx_index.unwrap_or_default()),
                        fees: details.fees,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to get details for tx {} ({}): {}. Using basic info.",
                        idx + 1,
                        &tx.tx_hash[..16],
                        e
                    );
                    // Use basic info from the list response as fallback
                    // Blockfrost list doesn't provide block hash or fees, so we use placeholders
                    transactions.push(crate::api::user::Transaction {
                        tx_hash: tx.tx_hash.clone(),
                        block: format!("block_{}", tx.block_height), // Placeholder block identifier
                        block_height: tx.block_height,
                        block_time: tx.block_time,
                        slot: 0, // Not available in list response
                        index: tx.tx_index.unwrap_or_default(),
                        fees: "0".to_string(), // Not available in list response
                    });
                }
            }
        }

        tracing::info!("Blockfrost: Successfully processed {} transactions", transactions.len());
        Ok(transactions)
    }

    async fn get_transaction_details(&self, tx_hash: &str) -> Result<BlockfrostTxDetails, String> {
        let url = format!("{}/txs/{}", self.base_url, tx_hash);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            tracing::error!("Blockfrost API error: {} - {}", status, text);
            return Err(format!("Blockfrost error: {} - {}", status, text));
        }

        let preview = if text.len() > 1000 { format!("{}... ({} bytes)", &text[..1000], text.len()) } else { text.clone() };
        serde_json::from_str(&text)
            .map_err(|e| {
                tracing::error!("Blockfrost JSON parse error: {}. Body: {}", e, preview);
                format!("Failed to parse response: {}. Body: {}", e, preview)
            })
    }

    pub async fn get_account_info(&self, address: &str) -> Result<crate::api::user::AccountInfo, String> {
        // Convert hex address to bech32 if needed (Blockfrost requires bech32)
        let bech32_address = normalize_address_for_blockfrost(address)
            .map_err(|e| format!("Address conversion failed: {}", e))?;
        
        // Detect network from address and warn if mismatch
        let address_network = detect_network_from_address(&bech32_address);
        let configured_network = if self.base_url.contains("mainnet") {
            "mainnet"
        } else if self.base_url.contains("preprod") {
            "preprod"
        } else if self.base_url.contains("preview") {
            "preview"
        } else {
            "unknown"
        };
        
        if address_network == "mainnet" && configured_network != "mainnet" {
            tracing::warn!(
                "⚠️  Network mismatch detected! Address is mainnet (addr1...) but Blockfrost is configured for {}",
                configured_network
            );
            tracing::warn!("   This will likely result in no data being returned. Consider using a {} address or configuring Blockfrost for mainnet.", configured_network);
        } else if address_network == "testnet" && configured_network == "mainnet" {
            tracing::warn!(
                "⚠️  Network mismatch detected! Address is testnet (addr_test...) but Blockfrost is configured for mainnet"
            );
        }
        
        tracing::info!(
            "Blockfrost: Converting address {} -> {}",
            &address[..address.len().min(16)],
            &bech32_address[..bech32_address.len().min(20)]
        );
        
        // Build URL for address info with proper URL encoding
        let base = reqwest::Url::parse(&self.base_url)
            .map_err(|e| format!("Invalid base URL: {}", e))?;
        
        // Use reqwest::Url::join() which handles URL encoding automatically
        let path_segment = format!("addresses/{}", bech32_address);
        let url = base.join(&path_segment)
            .map_err(|e| format!("Failed to build URL: {}", e))?;
        let url_str = url.as_str();

        tracing::info!("Blockfrost: Fetching account info");
        tracing::info!("  Original address: {} ({} chars)", &address[..address.len().min(32)], address.len());
        tracing::info!("  Bech32 address: {} ({} chars)", &bech32_address[..bech32_address.len().min(32)], bech32_address.len());
        tracing::info!("  URL: {}", url_str);

        let response = self
            .client
            .get(url_str)
            .header("project_id", &self.api_key)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            // Special-case: Blockfrost returns 404 when the address exists but has no on-chain data yet.
            if status.as_u16() == 404 {
                tracing::info!(
                    "Blockfrost: Address not found / no data (404) for {}; returning empty account info",
                    &bech32_address[..bech32_address.len().min(20)]
                );
                return Ok(crate::api::user::AccountInfo {
                    balance: "0".to_string(),
                    tx_count: 0,
                });
            }

            tracing::error!("Blockfrost API error: {} - {}", status, text);
            // Check if response is HTML (error page)
            if text.trim_start().starts_with("<!DOCTYPE") || text.trim_start().starts_with("<html") {
                return Err(format!(
                    "Blockfrost returned HTML instead of JSON (status: {}). This usually means:\n\
                    1. Invalid API key or API key not configured for this network\n\
                    2. Network mismatch between address and Blockfrost configuration\n\
                    3. Malformed request URL\n\
                    Address: {}...\n\
                    URL: {}",
                    status,
                    &bech32_address[..bech32_address.len().min(20)],
                    url_str
                ));
            }
            // Provide more helpful error messages
            if status == 400 {
                return Err(format!(
                    "Invalid request (400). Check address format.\n\
                    Address: {}...\n\
                    Error: {}",
                    &bech32_address[..bech32_address.len().min(20)],
                    text
                ));
            }
            return Err(format!("Blockfrost error: {} - {}", status, text));
        }

        // Check if response is HTML (shouldn't happen with 200 status, but just in case)
        if text.trim_start().starts_with("<!DOCTYPE") || text.trim_start().starts_with("<html") {
            tracing::error!("Blockfrost returned HTML instead of JSON even with success status");
            return Err(format!(
                "Blockfrost returned HTML instead of JSON. This suggests a configuration issue.\n\
                Address: {}...\n\
                URL: {}",
                &bech32_address[..bech32_address.len().min(20)],
                url_str
            ));
        }

        let preview = if text.len() > 1000 { format!("{}... ({} bytes)", &text[..1000], text.len()) } else { text.clone() };
        let info: BlockfrostAddressInfo = serde_json::from_str(&text)
            .map_err(|e| {
                tracing::error!("Failed to parse Blockfrost response: {}. Body: {}", e, preview);
                format!("Failed to parse response: {}. Body: {}", e, preview)
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