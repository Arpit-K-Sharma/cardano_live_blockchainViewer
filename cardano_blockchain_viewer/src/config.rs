// Configuration constants for the Cardano Blockchain Viewer

/// Number of events to keep in the circular buffer
pub const BUFFER_SIZE: usize = 100;

/// Maximum number of blocks before clearing the buffer
pub const MAX_BLOCK_COUNT: usize = 100;

/// Maximum number of transactions before clearing the buffer
pub const MAX_TX_COUNT: usize = 500;

/// Server listening address (for both REST API and WebSocket)
pub const SERVER_ADDR: &str = "127.0.0.1:8080";

/// Cardano network configuration
pub struct CardanoConfig {
    pub relay: &'static str,
    pub magic: Option<u64>,
    pub network_name: &'static str,
}


impl CardanoConfig {
    /// PreProd testnet configuration (default)
    pub fn preprod() -> Self {
        Self {
            relay: "preprod-node.world.dev.cardano.org:30000",
            magic: Some(1),
            network_name: "PreProd Testnet",
        }
    }

    /// Preview testnet configuration
    pub fn preview() -> Self {
        Self {

            relay: "preview-node.world.dev.cardano.org:3001",
            magic: Some(2),
            network_name: "Preview Testnet",
        }
    }

    /// Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            relay: "relays-new.cardano-mainnet.iohk.io:3001",
            magic: Some(3),
            network_name: "Mainnet",
        }
    }
}

impl Default for CardanoConfig {
    fn default() -> Self {
        Self::preprod()
    }
}