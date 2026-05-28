import type { FlatProcess, Tab } from "$types/metrics";

export function metricLabel(p: FlatProcess, activeTab: Tab): string {
  if (activeTab === "cpu") {
    if (p.cpu_percent <= 0) {
      return "0%";
    }
    if (p.cpu_percent < 0.1) {
      return "<0.1%";
    }
    return `${p.cpu_percent.toFixed(1)}%`;
  }
  if (activeTab === "ram") {
    return formatBytes(p.ram_bytes);
  }
  if (activeTab === "network") {
    if (p.net_bps <= 0) {
      return "0 B/s";
    }
    return `${formatBytes(p.net_bps)}/s`;
  }
  return "—";
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
  if (bytes >= 1_024) return `${(bytes / 1_024).toFixed(1)} KB`;
  return `${bytes} B`;
}


export function formatBps(bps: number): string {
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} MB/s`;
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} KB/s`;
  return `${bps} B/s`;
}
