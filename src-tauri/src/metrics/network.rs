use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

#[derive(Hash, Eq, PartialEq, Clone)]
struct SocketKey {
    local_port: u16,
    remote_addr: IpAddr,
    remote_port: u16,
}

struct PidSockets {
    established: u32,
    udp: u32,
    unique_remotes: HashSet<IpAddr>,
    socket_keys: HashSet<SocketKey>,
}

impl PidSockets {
    fn new() -> Self {
        Self {
            established: 0,
            udp: 0,
            unique_remotes: HashSet::new(),
            socket_keys: HashSet::new(),
        }
    }
}

pub struct NetTracker {
    prev_keys: HashMap<u32, HashSet<SocketKey>>,
    smoothed: HashMap<u32, f64>,
    decay_ticks: HashMap<u32, u8>,
}

impl NetTracker {
    pub fn new() -> Self {
        Self {
            prev_keys: HashMap::new(),
            smoothed: HashMap::new(),
            decay_ticks: HashMap::new(),
        }
    }

    pub fn update(&mut self, total_bps: u64) -> HashMap<u32, u64> {
        let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto = ProtocolFlags::TCP | ProtocolFlags::UDP;

        let sockets = match get_sockets_info(af, proto) {
            Ok(s) => s,
            Err(_) => return HashMap::new(),
        };

        let mut by_pid: HashMap<u32, PidSockets> = HashMap::new();

        for si in &sockets {
            match &si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp) => {
                    if tcp.state == TcpState::Listen {
                        continue;
                    }

                    let key = SocketKey {
                        local_port: tcp.local_port,
                        remote_addr: tcp.remote_addr,
                        remote_port: tcp.remote_port,
                    };

                    for &pid in &si.associated_pids {
                        let entry = by_pid.entry(pid).or_insert_with(PidSockets::new);
                        if tcp.state == TcpState::Established {
                            entry.established += 1;
                            entry.unique_remotes.insert(tcp.remote_addr);
                        }
                        entry.socket_keys.insert(key.clone());
                    }
                }

                ProtocolSocketInfo::Udp(_) => {
                    for &pid in &si.associated_pids {
                        by_pid.entry(pid).or_insert_with(PidSockets::new).udp += 1;
                    }
                }
            }
        }

        let mut scores: HashMap<u32, f64> = HashMap::with_capacity(by_pid.len());
        let empty: HashSet<SocketKey> = HashSet::new();

        for (pid, socks) in &by_pid {
            let prev = self.prev_keys.get(pid).unwrap_or(&empty);
            let new_conns = socks.socket_keys.difference(prev).count() as f64;
            let closed_conns = prev.difference(&socks.socket_keys).count() as f64;

            let score = socks.established as f64 * 1.0
                + socks.udp as f64 * 0.5
                + new_conns * 3.0
                + closed_conns * 2.0
                + socks.unique_remotes.len() as f64 * 0.5;

            if score > 0.0 {
                scores.insert(*pid, score);
            }
        }

        let total_score: f64 = scores.values().sum();
        let mut result: HashMap<u32, u64> = HashMap::with_capacity(scores.len());

        if total_score > 0.0 {
            for (pid, score) in &scores {
                let raw = (score / total_score) * total_bps as f64;
                let prev_smooth = self.smoothed.get(pid).copied().unwrap_or(raw);
                let smoothed = prev_smooth * 0.7 + raw * 0.3;
                self.smoothed.insert(*pid, smoothed);
                self.decay_ticks.remove(pid);
                result.insert(*pid, smoothed as u64);
            }
        }

        let mut to_remove: Vec<u32> = Vec::new();
        for (pid, smoothed) in self.smoothed.iter_mut() {
            if scores.contains_key(pid) {
                continue;
            }
            *smoothed *= 0.5;
            let ticks = self.decay_ticks.entry(*pid).or_insert(0);
            *ticks += 1;
            if *ticks >= 4 || *smoothed < 512.0 {
                to_remove.push(*pid);
            } else {
                result.insert(*pid, *smoothed as u64);
            }
        }
        for pid in &to_remove {
            self.smoothed.remove(pid);
            self.decay_ticks.remove(pid);
        }

        self.prev_keys = by_pid
            .into_iter()
            .map(|(pid, socks)| (pid, socks.socket_keys))
            .collect();

        result
    }
}
