use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const HISTORY_SIZE: usize = 60;

#[derive(Clone, Default)]
pub struct MetricsHistory {
    pub cpu: VecDeque<u8>,
    pub ram: VecDeque<u8>,
}

impl MetricsHistory {
    pub fn push(&mut self, cpu: u8, ram: u8) {
        push_capped(&mut self.cpu, cpu);
        push_capped(&mut self.ram, ram);
    }
}

fn push_capped(deque: &mut VecDeque<u8>, val: u8) {
    if deque.len() >= HISTORY_SIZE {
        deque.pop_front();
    }
    deque.push_back(val);
}

pub type SharedHistory = Arc<Mutex<MetricsHistory>>;

pub fn new_shared_history() -> SharedHistory {
    Arc::new(Mutex::new(MetricsHistory::default()))
}
