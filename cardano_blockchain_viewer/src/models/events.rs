use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OuraEvent {
    pub event: String,
    pub point: Point,
    pub record: Record,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point {
    pub hash: String,
    pub slot: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    // Here if the block is None then instead of passing block: None, It would rather omit the field entirely during serialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockRecord>,
        #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_input: Option<TxInputRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_output: Option<TxOutputRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roll_back: Option<RollBackRecord>,
    pub context: Context,
    pub fingerprint: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollBackRecord {
    pub block_hash: String,
    pub block_slot: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecord {
    pub hash: String,
    pub number: u64,
    pub slot: u64,
    pub epoch: u64,
    pub epoch_slot: u64,
    pub era: String,
    pub body_size: u32,
    pub issuer_vkey: String,
    pub vrf_vkey: String,
    pub tx_count: u32,
    pub previous_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub hash: String,
    pub fee: u64,
    pub size: u32,
    pub input_count: u32,
    pub output_count: u32,
    pub total_output: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_interval_start: Option<u64>,
    pub mint_count: u32,
    pub collateral_input_count: u32,
    pub has_collateral_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInputRecord {
    pub tx_id: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutputRecord {
    pub address: String,
    pub amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_idx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_idx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_idx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_idx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_address: Option<String>,
}


// ============================================================================
// Simplified Blockchain Events (Sent to frontend via WebSocket)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum BlockchainEvent {
    Block{
        slot: u64,
        hash: String,
        number: u64,
        epoch: u64,
        tx_count: u32,
        timestamp: u64,
        // Rest of the fields will be send by keeping inside the details so it will appear as a struct being passed
        #[serde(flatten)]
        details: serde_json::Value,
    },
    Transaction {
        hash: String,
        fee: u64,
        inputs: u32,
        outputs: u32,
        total_output: u64,
        timestamp: u64,
        #[serde(flatten)]
        details: serde_json::Value,
    },
    TxInput {
        tx_hash: String,
        input_tx_id: String,
        input_index: u32,
        timestamp: u64,
    },
    TxOutput {
        tx_hash: String,
        address: String,
        amount: u64,
        timestamp: u64,
    },
    RollBack {
        block_hash: String,
        block_slot: u64,
        timestamp: u64,
    },
    Other {
        event_type: String,
        timestamp: u64,
        #[serde(flatten)]
        details: serde_json::Value,
    },
}