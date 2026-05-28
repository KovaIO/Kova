use std::collections::HashMap;

use crate::processes::FlatProcess;

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
        .filter(|p| match p.parent_pid {
            None => true,
            Some(parent_pid) => match index_map.get(&parent_pid) {
                None => true,
                Some(parent_idx) => {
                    let parent_name = processes[*parent_idx].name.to_lowercase();
                    parent_name == "explorer.exe"
                }
            },
        })
        .map(|p| p.pid)
        .collect();

    for pid in roots {
        aggregate(pid, processes, &children, &index_map);
    }
}
