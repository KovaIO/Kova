use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const HISTORY_SIZE: usize = 60;

#[derive(Clone, Default)]
pub struct MetricsHistory {
    pub cpu: VecDeque<u8>,
    pub ram: VecDeque<u8>,
    pub network: VecDeque<u64>,
}

impl MetricsHistory {
    pub fn push(&mut self, cpu: u8, ram: u8, network: u64) {
        push_capped_u8(&mut self.cpu, cpu);
        push_capped_u8(&mut self.ram, ram);
        push_capped_u64(&mut self.network, network);
    }
}

fn push_capped_u8(deque: &mut VecDeque<u8>, val: u8) {
    if deque.len() >= HISTORY_SIZE {
        deque.pop_front();
    }
    deque.push_back(val);
}

fn push_capped_u64(deque: &mut VecDeque<u64>, val: u64) {
    if deque.len() >= HISTORY_SIZE {
        deque.pop_front();
    }
    deque.push_back(val);
}

pub type SharedHistory = Arc<Mutex<MetricsHistory>>;

pub fn new_shared_history() -> SharedHistory {
    Arc::new(Mutex::new(MetricsHistory::default()))
}
