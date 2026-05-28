export type Tab = "cpu" | "ram" | "disk" | "network";

export interface Metrics {
  cpu_percent: number;
  ram_used: number;
  ram_total: number;
  ram_percent: number;
  disk_used: number;
  disk_total: number;
  disk_percent: number;
  network_bps: number;
  cpu_history: number[];
  ram_history: number[];
  network_history: number[];
  processes: FlatProcess[];
}

export interface FlatProcess {
  pid: number;
  parent_pid?: number | null;

  started_at: number;
  exe_path: string;

  name: string;

  cpu_percent: number;
  ram_bytes: number;
  net_bps: number;

  icon?: string | null;
}

export interface ProcessNode extends FlatProcess {
  children: ProcessNode[];
}
