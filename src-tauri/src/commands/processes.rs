use serde::Serialize;
use std::collections::HashMap;
use sysinfo::System;
use tauri::command;

use crate::commands::process_icons::get_process_icon;

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

#[derive(Serialize)]
pub struct RunningProcess {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[command]
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

            FlatProcess {
                pid,
                parent_pid: proc.parent().map(|p| p.as_u32()),

                name: meta.name.clone(),

                exe_path: meta.exe_path.clone(),
                started_at: meta.started_at,

                self_cpu_percent: proc.cpu_usage(),
                self_ram_bytes: proc.memory(),
                self_net_bps: net_bps,

                cpu_percent: proc.cpu_usage(),
                ram_bytes: proc.memory(),
                net_bps,

                icon: meta.icon.clone(),
            }
        })
        .collect::<Vec<_>>();

    metadata_cache.retain(|pid, _| alive.contains_key(pid));

    processes
}

pub fn aggregate_process_metrics(processes: &mut Vec<FlatProcess>) {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut index_map: HashMap<u32, usize> = HashMap::new();

    for (idx, proc) in processes.iter().enumerate() {
        index_map.insert(proc.pid, idx);
    }

    for proc in processes.iter() {
        if let Some(parent) = proc.parent_pid {
            let parent_idx = index_map.get(&parent).copied();

            if let Some(idx) = parent_idx {
                let parent_name = processes[idx].name.to_lowercase();

                if parent_name != "explorer.exe" {
                    children.entry(parent).or_default().push(proc.pid);
                }
            }
        }
    }

    fn aggregate(
        pid: u32,
        processes: &mut Vec<FlatProcess>,
        children: &HashMap<u32, Vec<u32>>,
        index_map: &HashMap<u32, usize>,
    ) -> (f32, u64, u64) {
        let idx = index_map[&pid];

        let mut cpu = processes[idx].self_cpu_percent;
        let mut ram = processes[idx].self_ram_bytes;
        let mut net = processes[idx].self_net_bps;

        if let Some(childs) = children.get(&pid) {
            for child_pid in childs {
                let (c_cpu, c_ram, c_net) = aggregate(*child_pid, processes, children, index_map);

                cpu += c_cpu;
                ram += c_ram;
                net += c_net;
            }
        }

        processes[idx].cpu_percent = cpu;
        processes[idx].ram_bytes = ram;
        processes[idx].net_bps = net;

        (cpu, ram, net)
    }

    let roots: Vec<u32> = processes
        .iter()
        .filter(|p| p.parent_pid.is_none() || !index_map.contains_key(&p.parent_pid.unwrap()))
        .map(|p| p.pid)
        .collect();

    for pid in roots {
        aggregate(pid, processes, &children, &index_map);
    }
}
