use std::{collections::HashMap, time::Duration};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

use crate::{
    metrics::{
        disk::{find_disk_index, primary_disk_metrics},
        models::{HistoricalSnapshot, Metrics, SharedHistory, SystemMetrics},
        network::NetTracker,
    },
    processes::{aggregate_process_metrics, collect_processes, ProcessMeta, ProcessSnapshot},
};

#[cfg(target_os = "windows")]
use crate::metrics::disk::windows_system_drive;

pub fn start_metrics_loop(app: AppHandle, history: SharedHistory) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        let mut disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        let mut net_tracker = NetTracker::new();

        let mut metadata_cache = HashMap::<u32, ProcessMeta>::new();

        sys.refresh_cpu_all();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

        #[cfg(target_os = "windows")]
        let target = std::path::PathBuf::from(windows_system_drive());

        #[cfg(not(target_os = "windows"))]
        let target = std::path::PathBuf::from("/");

        let disk_index = find_disk_index(&disks, &target);

        let mut prev_network_total: u64 = 0;
        const INTERVAL_SECS: u64 = 5;

        loop {
            sys.refresh_cpu_all();
            sys.refresh_memory();
            sys.refresh_processes(ProcessesToUpdate::All, true);

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

            let net_map = net_tracker.update(network_bps);

            let mut processes = collect_processes(&sys, &net_map, &mut metadata_cache);
            aggregate_process_metrics(&mut processes);

            let snapshot_processes: Vec<ProcessSnapshot> =
                processes.iter().map(ProcessSnapshot::from).collect();

            let historical = HistoricalSnapshot {
                processes: snapshot_processes,
            };

            let (cpu_history, ram_history, network_history) = {
                let mut h = history.lock().unwrap();

                h.push(
                    cpu_percent,
                    ram_percent,
                    ram_total,
                    network_bps,
                    historical,
                    metadata_cache.clone(),
                );

                (
                    h.cpu.iter().copied().collect::<Vec<_>>(),
                    h.ram.iter().copied().collect::<Vec<_>>(),
                    h.network.iter().copied().collect::<Vec<_>>(),
                )
            };

            let system_metrics = SystemMetrics {
                cpu_percent,
                ram_used,
                ram_total,
                ram_percent,
                disk_used,
                disk_total,
                disk_percent,
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
                processes,
            };

            if let Err(err) = app.emit("system-metrics", &system_metrics) {
                eprintln!("failed to emit system-metrics: {err}");
            }

            if let Err(err) = app.emit("metrics", &metrics) {
                eprintln!("failed to emit metrics: {err}");
            }

            tokio::time::sleep(Duration::from_secs(INTERVAL_SECS)).await;
        }
    });
}
