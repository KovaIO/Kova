use crate::app_state::SharedHistory;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;
use sysinfo::{Disks, Networks, System};
use tauri::{AppHandle, Emitter};

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
}

pub fn start_metrics_loop(app: AppHandle, history: SharedHistory) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        let mut disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();

        sys.refresh_cpu_all();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

        #[cfg(target_os = "windows")]
        let target = std::path::PathBuf::from(windows_system_drive());

        #[cfg(not(target_os = "windows"))]
        let target = std::path::PathBuf::from("/");

        let disk_index = find_disk_index(&disks, &target);

        let mut prev_network_total: u64 = 0;
        const INTERVAL_SECS: u64 = 2;

        loop {
            sys.refresh_cpu_all();
            sys.refresh_memory();

            networks.refresh(false);
            disks.refresh(false);

            let cpu_percent = sys.global_cpu_usage().round() as u8;

            let ram_used = sys.used_memory();
            let ram_total = sys.total_memory();
            let ram_percent = if ram_total > 0 {
                ((ram_used as f32 / ram_total as f32) * 100.0).round() as u8
            } else {
                0
            };

            let (disk_used, disk_total, disk_percent) = primary_disk_metrics(&disks, disk_index);

            let current_total: u64 = networks
                .iter()
                .map(|(_, n)| n.received() + n.transmitted())
                .sum();

            let network_bps = current_total.saturating_sub(prev_network_total) / INTERVAL_SECS;
            prev_network_total = current_total;

            let (cpu_history, ram_history, network_history) = {
                let mut h = history.lock().unwrap();
                h.push(cpu_percent, ram_percent, network_bps);
                (
                    h.cpu.iter().copied().collect::<Vec<_>>(),
                    h.ram.iter().copied().collect::<Vec<_>>(),
                    h.network.iter().copied().collect::<Vec<_>>(),
                )
            };

            let metrics = Metrics {
                cpu_percent,
                ram_used,
                ram_total,
                ram_percent,
                disk_used,
                disk_total,
                disk_percent,
                network_bps,
                cpu_history,
                ram_history,
                network_history,
            };

            if let Err(err) = app.emit("metrics", &metrics) {
                eprintln!("failed to emit metrics: {err}");
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn find_disk_index(disks: &Disks, target: &Path) -> Option<usize> {
    disks.list().iter().position(|d| d.mount_point() == target)
}

fn primary_disk_metrics(disks: &Disks, index: Option<usize>) -> (u64, u64, u8) {
    let disk = index.and_then(|i| disks.list().get(i));
    match disk {
        Some(disk) => {
            let total = disk.total_space();
            let used = total.saturating_sub(disk.available_space());
            let percent = if total > 0 {
                ((used as f32 / total as f32) * 100.0).round() as u8
            } else {
                0
            };
            (used, total, percent)
        }
        None => (0, 0, 0),
    }
}

#[cfg(target_os = "windows")]
fn windows_system_drive() -> String {
    std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()) + "\\"
}

#[tauri::command]
pub fn get_current_metrics(history: tauri::State<SharedHistory>) -> Option<Metrics> {
    let h = history.lock().unwrap();
    if h.cpu.is_empty() {
        return None;
    }
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
    })
}
