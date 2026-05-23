use serde::Serialize;
use std::path::Path;
use std::time::Duration;
use sysinfo::{Disks, System};
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
}

pub fn start_metrics_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        let mut disks = Disks::new_with_refreshed_list();

        sys.refresh_cpu_all();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

        #[cfg(target_os = "windows")]
        let target = std::path::PathBuf::from(windows_system_drive());

        #[cfg(not(target_os = "windows"))]
        let target = std::path::PathBuf::from("/");

        loop {
            sys.refresh_cpu_all();
            sys.refresh_memory();

            disks.refresh(true);

            let cpu_percent = sys.global_cpu_usage().round() as u8;

            let ram_used = sys.used_memory();
            let ram_total = sys.total_memory();
            let ram_percent = if ram_total > 0 {
                ((ram_used as f32 / ram_total as f32) * 100.0).round() as u8
            } else {
                0
            };

            let (disk_used, disk_total, disk_percent) = primary_disk_metrics(&disks, &target);

            let metrics = Metrics {
                cpu_percent,
                ram_used,
                ram_total,
                ram_percent,
                disk_used,
                disk_total,
                disk_percent,
            };

            if let Err(err) = app.emit("metrics", &metrics) {
                eprintln!("failed to emit metrics: {err}");
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn primary_disk_metrics(disks: &Disks, target: &Path) -> (u64, u64, u8) {
    for disk in disks.list() {
        if disk.mount_point() == target {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            let percent = if total > 0 {
                ((used as f32 / total as f32) * 100.0).round() as u8
            } else {
                0
            };

            return (used, total, percent);
        }
    }

    (0, 0, 0)
}

#[cfg(target_os = "windows")]
fn windows_system_drive() -> String {
    std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()) + "\\"
}
