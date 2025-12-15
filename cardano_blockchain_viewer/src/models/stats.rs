use serde::Serialize;
use std::collections::VecDeque;

use super::BlockchainEvent;

/// Statistics about buffered blockchain events
#[derive(Debug, Clone, Serialize)]
pub struct BufferStats {
    // usize is used for array and vector indexing, .len() and .capacity()
    pub total_events: usize,
    pub blocks_count: usize,
    pub transactions_count: usize,
    pub inputs_count: usize,
    pub outputs_count: usize,
    pub buffer_size: usize,
    pub last_block_number: u64,
    pub last_slot: u64,
}

/// Application state holding the event buffer and statistics
pub struct AppState {
    pub buffer: VecDeque<BlockchainEvent>,
    pub blocks_count: usize,
    pub transactions_count: usize,
    pub inputs_count: usize,
    pub outputs_count: usize,
    pub total_events: usize,
    pub last_block_number: u64,
    pub last_slot: u64,
}

impl AppState {
    // Create a new AppState with the given buffer capacity
    pub fn new(capacity: usize) -> Self {
        Self{
            buffer: VecDeque::with_capacity(capacity),
            blocks_count: 0,
            transactions_count: 0,
            inputs_count: 0,
            outputs_count: 0,
            total_events: 0,
            last_block_number: 0,
            last_slot: 0,
        }
    }

    // Add an event to the buffer and update statistics
    pub fn add_event(&mut self, event: BlockchainEvent, buffer_size: usize){

        // Count event types
        // the match event check which type is it from the enum in the events.rs
        // If the BlockchainEvent is block it runs the written code
        // It checks which type is it and then increases the count
        match &event {
            // use only number and slot and ignore other field
            BlockchainEvent::Block { number, slot, ..} => {
                self.blocks_count += 1;
                self.last_block_number = *number;
                self.last_slot = *slot;
            }
            BlockchainEvent::Transaction { .. } => self.transactions_count += 1,
            BlockchainEvent::TxInput { .. } => self.inputs_count += 1,
            BlockchainEvent::TxOutput { .. } => self.outputs_count += 1,
            // _ = catch-all pattern.
            // Matches any value that hasn’t been matched earlier.
            // {} = do nothing.
            _ => {}
        }

        self.total_events += 1;

        // Add to buffer (circular buffer)
        if self.buffer.len() >= buffer_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(event);
    }

    // Clear the buffer and reset counters (but keep total_events)
    pub fn clear_buffer(&mut self){
        self.buffer.clear();
        self.blocks_count = 0;
        self.transactions_count = 0;
        self.inputs_count = 0;
        self.outputs_count = 0;
    }

    /// Check if buffer should be cleared based on thresholds
    pub fn should_clear(&self, max_blocks: usize, max_txs: usize) -> bool {
        self.blocks_count >= max_blocks || self.transactions_count >= max_txs
    }

    // Get current statistics
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            total_events: self.total_events,
            blocks_count: self.blocks_count,
            transactions_count: self.transactions_count,
            inputs_count: self.inputs_count,
            outputs_count: self.outputs_count,
            buffer_size: self.buffer.len(),
            last_block_number: self.last_block_number,
            last_slot: self.last_slot,
        }
    }
}