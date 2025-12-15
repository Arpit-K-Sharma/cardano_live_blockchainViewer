use reqwest::Client;
use serde::{Deserialize, Serialize};

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
pub struct AccountInfo {
    pub controlled_amount: String,
    pub stake_address: Option<String>,
    pub tx_count: usize,
}

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
        let url = format!("{}/addresses/{}/transactions", self.base_url, address);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .query(&[("page", page.to_string()), ("count", count.to_string())])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Blockfrost error: {}", response.status()));
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
            return Err(format!("Blockfrost error: {}", response.status()));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_account_info(&self, address: &str) -> Result<crate::api::user::AccountInfo, String> {
        let url = format!("{}/addresses/{}", self.base_url, address);

        let response = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Blockfrost error: {}", response.status()));
        }

        let info: AccountInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(crate::api::user::AccountInfo {
            balance: info.controlled_amount,
            tx_count: info.tx_count,
        })
    }
}