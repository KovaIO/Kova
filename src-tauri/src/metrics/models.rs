use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::processes::{FlatProcess, ProcessMeta, ProcessSnapshot};

#[derive(Serialize)]
pub struct SystemMetrics {
    pub cpu_percent: u8,
    pub ram_used: u64,
    pub ram_total: u64,
    pub ram_percent: u8,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_percent: u8,
}

#[derive(Serialize)]
pub struct Metrics {
    pub cpu_percent: u8,

    pub ram_used: u64,
    pub ram_total: u64,
    pub ram_percent: u8,

    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_percent: u8,

    pub network_bps: u64,

    pub cpu_history: Vec<u8>,
    pub ram_history: Vec<u8>,
    pub network_history: Vec<u64>,
    pub processes: Vec<FlatProcess>,
}

#[derive(Clone)]
pub struct HistoricalSnapshot {
    pub processes: Vec<ProcessSnapshot>,
}

pub const HISTORY_SIZE: usize = 60;

#[derive(Clone, Default)]
pub struct MetricsHistory {
    pub cpu: VecDeque<u8>,
    pub ram: VecDeque<u8>,
    pub last_ram_total: u64,
    pub network: VecDeque<u64>,
    pub process_history: VecDeque<HistoricalSnapshot>,
    pub process_metadata: HashMap<u32, ProcessMeta>,
}

impl MetricsHistory {
    pub fn push(
        &mut self,
        cpu: u8,
        ram: u8,
        ram_total: u64,
        network: u64,
        snapshot: HistoricalSnapshot,
        metadata: HashMap<u32, ProcessMeta>,
    ) {
        push_capped_u8(&mut self.cpu, cpu);
        push_capped_u8(&mut self.ram, ram);
        push_capped_u64(&mut self.network, network);
        if self.process_history.len() >= HISTORY_SIZE {
            self.process_history.pop_front();
        }

        self.process_history.push_back(snapshot);
        self.process_metadata = metadata;
        self.last_ram_total = ram_total;
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
