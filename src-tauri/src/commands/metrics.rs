use crate::{
    metrics::{models::SharedHistory, Metrics},
    processes::FlatProcess,
};

#[tauri::command]
pub fn get_current_metrics(history: tauri::State<SharedHistory>) -> Option<Metrics> {
    let h = history.lock().unwrap();
    if h.cpu.is_empty() {
        return None;
    }

    let ram_total = h.last_ram_total;
    let ram_percent = *h.ram.back().unwrap_or(&0);
    let ram_used = (ram_percent as f64 / 100.0 * ram_total as f64) as u64;

    let processes: Vec<FlatProcess> = h
        .process_history
        .back()
        .map(|s| s.processes.clone())
        .unwrap_or_default();

    Some(Metrics {
        cpu_percent: *h.cpu.back().unwrap_or(&0),
        ram_percent,
        network_bps: *h.network.back().unwrap_or(&0),
        ram_used,
        ram_total,
        disk_used: 0,
        disk_total: 0,
        disk_percent: 0,
        cpu_history: h.cpu.iter().copied().collect(),
        ram_history: h.ram.iter().copied().collect(),
        network_history: h.network.iter().copied().collect(),
        processes,
    })
}
