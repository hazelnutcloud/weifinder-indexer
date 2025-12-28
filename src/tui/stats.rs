use std::sync::{Arc, atomic::AtomicU64};

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub last_saved_block: Arc<AtomicU64>,
    pub last_fetched_block: Arc<AtomicU64>,
    pub current_head_number: Arc<AtomicU64>,
}

impl Stats {
    pub fn set_last_saved_block(&mut self, last_saved_block: Arc<AtomicU64>) {
        self.last_saved_block = last_saved_block;
    }

    pub fn set_last_fetched_block(&mut self, last_fetched_block: Arc<AtomicU64>) {
        self.last_fetched_block = last_fetched_block;
    }

    pub fn set_current_head_number(&mut self, current_head_number: Arc<AtomicU64>) {
        self.current_head_number = current_head_number;
    }
}
