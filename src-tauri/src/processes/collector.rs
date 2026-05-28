use std::collections::HashMap;

use sysinfo::System;

use crate::processes::{get_process_icon, FlatProcess, ProcessMeta, RunningProcess};

pub fn get_running_processes() -> Vec<RunningProcess> {
    let sys = System::new_all();
    let mut processes = Vec::new();

    for (_, proc) in sys.processes() {
        if let Some(exe_path) = proc.exe() {
            let path = exe_path.to_string_lossy().to_string();
            let name = proc.name().to_string_lossy().to_string();
            let icon = get_process_icon(&name, Some(&path));

            // Only include processes with valid exe paths
            if !path.is_empty() && !name.is_empty() {
                // Filter out Windows system processes
                let path_lower = path.to_lowercase();
                let is_system = path_lower.contains("\\windows\\system32\\")
                    || path_lower.contains("\\windows\\syswow64\\")
                    || path_lower.contains("\\windows\\system\\")
                    || name.to_lowercase() == "system"
                    || name.to_lowercase() == "system idle process"
                    || name.to_lowercase() == "registry";

                if !is_system {
                    processes.push(RunningProcess { name, path, icon });
                }
            }
        }
    }

    processes.sort_by(|a, b| a.name.cmp(&b.name));
    processes.dedup_by(|a, b| a.path == b.path);

    processes
}

pub fn collect_processes(
    sys: &System,
    net_map: &HashMap<u32, u64>,
    metadata_cache: &mut HashMap<u32, ProcessMeta>,
) -> Vec<FlatProcess> {
    let mut alive = HashMap::<u32, bool>::new();

    let processes = sys
        .processes()
        .iter()
        .map(|(pid, proc)| {
            let pid = pid.as_u32();

            alive.insert(pid, true);

            let meta = metadata_cache.entry(pid).or_insert_with(|| {
                let exe_path = proc.exe().map(|p| p.to_string_lossy().to_string());

                let icon =
                    get_process_icon(proc.name().to_string_lossy().as_ref(), exe_path.as_deref());

                ProcessMeta {
                    name: proc.name().to_string_lossy().into_owned(),
                    icon,
                    started_at: proc.start_time(),
                    exe_path: proc.exe().map(|p| p.to_string_lossy().to_string()),
                }
            });

            let net_bps = net_map.get(&pid).copied().unwrap_or(0);

            let cpu_cores = sys.cpus().len().max(1) as f32;

            FlatProcess {
                pid,
                parent_pid: proc.parent().map(|p| p.as_u32()),

                name: meta.name.clone(),

                exe_path: meta.exe_path.clone(),
                started_at: meta.started_at,

                self_cpu_percent: proc.cpu_usage() / cpu_cores,
                self_ram_bytes: proc.memory(),
                self_net_bps: net_bps,

                cpu_percent: proc.cpu_usage() / cpu_cores,
                ram_bytes: proc.memory(),
                net_bps,

                icon: meta.icon.clone(),
            }
        })
        .collect::<Vec<_>>();

    metadata_cache.retain(|pid, _| alive.contains_key(pid));

    processes
}
