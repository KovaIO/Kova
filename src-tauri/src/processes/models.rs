use serde::Serialize;

#[derive(Clone)]
pub struct ProcessMeta {
    pub name: String,
    pub icon: Option<String>,
    pub exe_path: Option<String>,
    pub started_at: u64,
}

#[derive(Serialize, Clone)]
pub struct FlatProcess {
    pub pid: u32,
    pub parent_pid: Option<u32>,

    pub name: String,

    pub exe_path: Option<String>,
    pub started_at: u64,

    pub self_cpu_percent: f32,
    pub self_ram_bytes: u64,
    pub self_net_bps: u64,

    pub cpu_percent: f32,
    pub ram_bytes: u64,
    pub net_bps: u64,

    pub icon: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub self_cpu_percent: f32,
    pub self_ram_bytes: u64,
    pub self_net_bps: u64,
    pub cpu_percent: f32,
    pub ram_bytes: u64,
    pub net_bps: u64,
}

impl From<&FlatProcess> for ProcessSnapshot {
    fn from(process: &FlatProcess) -> Self {
        Self {
            pid: process.pid,
            parent_pid: process.parent_pid,
            self_cpu_percent: process.self_cpu_percent,
            self_ram_bytes: process.self_ram_bytes,
            self_net_bps: process.self_net_bps,
            cpu_percent: process.cpu_percent,
            ram_bytes: process.ram_bytes,
            net_bps: process.net_bps,
        }
    }
}
