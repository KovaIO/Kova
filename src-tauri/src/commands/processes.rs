use crate::{
    metrics::models::SharedHistory,
    processes::{force_quit_process, get_running_processes, quit_process, RunningProcess},
};

#[tauri::command]
pub fn get_running_procs() -> Vec<RunningProcess> {
    get_running_processes()
}

#[tauri::command]
pub fn get_process_history(
    pid: u32,
    history: tauri::State<SharedHistory>,
) -> (Vec<f32>, Vec<u64>, Vec<u64>) {
    let h = history.lock().unwrap();

    let cpu: Vec<f32> = h
        .process_history
        .iter()
        .filter_map(|snap| snap.processes.iter().find(|p| p.pid == pid))
        .map(|p| p.cpu_percent)
        .collect();

    let ram: Vec<u64> = h
        .process_history
        .iter()
        .filter_map(|snap| snap.processes.iter().find(|p| p.pid == pid))
        .map(|p| p.ram_bytes)
        .collect();

    let net: Vec<u64> = h
        .process_history
        .iter()
        .filter_map(|snap| snap.processes.iter().find(|p| p.pid == pid))
        .map(|p| p.net_bps)
        .collect();

    (cpu, ram, net)
}

#[tauri::command]
pub fn quit_process_cmd(pid: u32) -> Result<(), String> {
    quit_process(pid)
}

#[tauri::command]
pub fn force_quit_process_cmd(pid: u32) -> Result<(), String> {
    force_quit_process(pid)
}
