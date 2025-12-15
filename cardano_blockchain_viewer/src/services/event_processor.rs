// Let's multiple part of your program share the same data safely
use std::sync::Arc;
// It makes sure only one task can modify data at a time
use tokio::sync::{Mutex, broadcast};
use tracing::info;

use crate::config::{BUFFER_SIZE, MAX_BLOCK_COUNT, MAX_TX_COUNT};
use crate::models::{AppState, BlockchainEvent, OuraEvent};

// Service for processing Oura events and managing application state
pub struct EventProcessor {
    state: Arc<Mutex<AppState>>,
}

impl EventProcessor {
    // Create a new EventProcessor with shared state
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self { state }
    }

    // Process an Oura event: convert it, update state and broadcast
    pub async fn process_event(
        &self,
        oura_event: OuraEvent,
        ws_tx: &broadcast::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert Oura event to simplified blockchain event
        let event = self.convert_oura_event(oura_event);

        // Log Summary
        self.log_event(&event);

        // Add to buffer and update state
        {
            let mut state = self.state.lock().await;
            state.add_event(event.clone(), BUFFER_SIZE);

            // Check if we should clear the buffer
            if state.should_clear(MAX_BLOCK_COUNT, MAX_TX_COUNT) {
                info!(
                    "Clearing buffer: blocks={}, txs={}",
                    state.blocks_count, state.transactions_count
                );
                state.clear_buffer();
            }

            // Send stats immediately for Block and Transaction events, and every 5 events for others
            let should_send_stats = match &event {
                BlockchainEvent::Block { .. } | BlockchainEvent::Transaction { .. } => true,
                _ => state.total_events % 5 == 0,
            };

            if should_send_stats {
                let stats = state.get_stats();
                info!(
                    "📊 Stats: blocks={}, txs={}, inputs={}, outputs={}, total={}",
                    stats.blocks_count,
                    stats.transactions_count,
                    stats.inputs_count,
                    stats.outputs_count,
                    stats.total_events
                );

                // Create the expected stats message format for the frontend
                let stats_message = serde_json::json!({
                    "type": "stats",
                    "data": stats
                });

                let stats_json = serde_json::to_string(&stats_message)?;

                // Check if there are any active receivers before sending
                if ws_tx.receiver_count() > 0 {
                    if let Err(e) = ws_tx.send(stats_json) {
                        // Channel send failed, but we have receivers so this is unexpected
                        if state.total_events % 50 == 0 {
                            info!("Failed to send stats to active receivers: {}", e);
                        }
                    }
                }
            }
        }

        // Broadcast to WebSocket clients
        let event_json = serde_json::to_string(&event)?;

        if let Err(e) = ws_tx.send(event_json) {
            // Channel is likely full or closed (no receivers)
            // This is normal when no WebSocket clients are connected
            // Silent failure to avoid log spam
        }

        Ok(())
    }

    // Convert Oura event to simplified blockchain event
    fn convert_oura_event(&self, oura_event: OuraEvent) -> BlockchainEvent {
        let timestamp = oura_event.record.context.timestamp.unwrap_or(0);

        // Check what type of record this is
        if let Some(block) = oura_event.record.block {
            BlockchainEvent::Block {
                slot: block.slot,
                hash: block.hash.clone(),
                number: block.number,
                epoch: block.epoch,
                tx_count: block.tx_count,
                timestamp,
                details: serde_json::to_value(&block).unwrap_or(serde_json::Value::Null),
            }
        } else if let Some(tx) = oura_event.record.transaction {
            BlockchainEvent::Transaction {
                hash: tx.hash.clone(),
                fee: tx.fee,
                inputs: tx.input_count,
                outputs: tx.output_count,
                total_output: tx.total_output,
                timestamp,
                details: serde_json::to_value(&tx).unwrap_or(serde_json::Value::Null),
            }
        } else if let Some(input) = oura_event.record.tx_input {
            BlockchainEvent::TxInput {
                tx_hash: oura_event.record.context.tx_hash.unwrap_or_default(),
                input_tx_id: input.tx_id,
                input_index: input.index,
                timestamp,
            }
        } else if let Some(output) = oura_event.record.tx_output {
            BlockchainEvent::TxOutput {
                tx_hash: oura_event.record.context.tx_hash.unwrap_or_default(),
                address: output.address,
                amount: output.amount,
                timestamp,
            }
        } else if let Some(rollback) = oura_event.record.roll_back {
            BlockchainEvent::RollBack {
                block_hash: rollback.block_hash,
                block_slot: rollback.block_slot,
                timestamp,
            }
        } else {
            let event_type = oura_event.event.clone();
            BlockchainEvent::Other {
                event_type,
                timestamp,
                details: serde_json::to_value(&oura_event).unwrap_or(serde_json::Value::Null),
            }
        }
    }

    // Log a summary of the blockchain event
    fn log_event(&self, event: &BlockchainEvent) {
        match event {
            // If event type is Block
            BlockchainEvent::Block {
                number,
                slot,
                tx_count,
                ..
            } => {
                info!(
                    "📦 Block #{} at slot {} with {} transactions",
                    number, slot, tx_count
                );
            }

            // If event type is transaction
            BlockchainEvent::Transaction { hash, fee, .. } => {
                info!("💳 Transaction {} (fee: {} lovelace)", &hash[..16], fee);
            }

            // If event type is TxInput
            BlockchainEvent::TxInput {
                input_tx_id,
                input_index,
                ..
            } => {
                info!("📥 Input: {}:{}", &input_tx_id[..16], input_index);
            }

            // If event type is TxOutput
            BlockchainEvent::TxOutput {
                address, amount, ..
            } => {
                info!("📤 Output: {} lovelace to {}", amount, &address[..20]);
            }

            BlockchainEvent::RollBack {
                block_hash,
                block_slot,
                ..
            } => {
                info!(
                    "🔄 Rollback to block {} at slot {}",
                    &block_hash[..16],
                    block_slot
                );
            }

            // If none then:
            _ => {}
        }
    }

    // Get the current Application state (for WebSocket initial sync)
    pub fn get_state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.state)
    }
}
