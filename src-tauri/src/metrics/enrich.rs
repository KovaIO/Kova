use std::collections::HashMap;

use crate::processes::{get_process_icon, FlatProcess, ProcessMeta, ProcessSnapshot};

pub fn enrich_process_snapshots(
    snapshots: &[ProcessSnapshot],
    metadata: &HashMap<u32, ProcessMeta>,
) -> Vec<FlatProcess> {
    snapshots
        .iter()
        .map(|snapshot| enrich_process_snapshot(snapshot, metadata))
        .collect()
}

pub fn enrich_process_snapshot(
    snapshot: &ProcessSnapshot,
    metadata: &HashMap<u32, ProcessMeta>,
) -> FlatProcess {
    if let Some(meta) = metadata.get(&snapshot.pid) {
        return FlatProcess {
            pid: snapshot.pid,
            parent_pid: snapshot.parent_pid,
            name: meta.name.clone(),
            exe_path: meta.exe_path.clone(),
            started_at: meta.started_at,
            icon: meta.icon.clone(),
            self_cpu_percent: snapshot.self_cpu_percent,
            self_ram_bytes: snapshot.self_ram_bytes,
            self_net_bps: snapshot.self_net_bps,
            cpu_percent: snapshot.cpu_percent,
            ram_bytes: snapshot.ram_bytes,
            net_bps: snapshot.net_bps,
        };
    }

    let name = format!("PID {}", snapshot.pid);

    FlatProcess {
        pid: snapshot.pid,
        parent_pid: snapshot.parent_pid,
        name: name.clone(),
        exe_path: None,
        started_at: 0,
        icon: get_process_icon(&name, None),
        self_cpu_percent: snapshot.self_cpu_percent,
        self_ram_bytes: snapshot.self_ram_bytes,
        self_net_bps: snapshot.self_net_bps,
        cpu_percent: snapshot.cpu_percent,
        ram_bytes: snapshot.ram_bytes,
        net_bps: snapshot.net_bps,
    }
}
