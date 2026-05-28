use crate::{
    metrics::{models::SharedHistory, Metrics},
    processes::{get_running_processes, FlatProcess, RunningProcess},
};

#[tauri::command]
pub fn get_current_metrics(history: tauri::State<SharedHistory>) -> Option<Metrics> {
    let h = history.lock().unwrap();
    if h.cpu.is_empty() {
        return None;
    }

    let processes: Vec<FlatProcess> = h
        .process_history
        .back()
        .map(|s| s.processes.clone())
        .unwrap_or_default();

    Some(Metrics {
        cpu_percent: *h.cpu.back().unwrap_or(&0),
        ram_percent: *h.ram.back().unwrap_or(&0),
        network_bps: *h.network.back().unwrap_or(&0),
        ram_used: 0,
        ram_total: 0,
        disk_used: 0,
        disk_total: 0,
        disk_percent: 0,
        cpu_history: h.cpu.iter().copied().collect(),
        ram_history: h.ram.iter().copied().collect(),
        network_history: h.network.iter().copied().collect(),
        processes,
    })
}

#[tauri::command]
pub fn get_running_procs() -> Vec<RunningProcess> {
    get_running_processes()
}
