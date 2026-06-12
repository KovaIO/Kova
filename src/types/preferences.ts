export interface Preferences {
  general: GeneralPreferences;
  clipboard: ClipboardPreferences;
  appearance: AppearancePreferences;
  power: PowerPreferences;
  shortcuts: Shortcut[];
}

export interface GeneralPreferences {
  launch_at_startup: boolean;
  show_menu_bar: boolean;
  language: string;
  monitor_dim: number;
}

export interface ClipboardPreferences {
  enabled: boolean;
  history_limit: number;
  ignore_passwords: boolean;
  ignored_apps: IgnoredApp[];
}

export type AccentColor = "blue" | "purple" | "green" | "orange" | "white";

export type WindowDensity = "normal" | "wide";

export type MetricCardStyle = "block" | "ring";

export interface AppearancePreferences {
  accent_color: AccentColor;
  window_density: WindowDensity;
  metric_card_style: MetricCardStyle;
}

export interface PowerPreferences {
  reduce_cpu_when_idle: boolean;
  disable_animations_on_battery: boolean;
  idle_timeout_seconds: number;
}

export interface IgnoredApp {
  name: string;
  path: string;
  exe_path?: string;
  icon?: string;
}

export interface Shortcut {
  action: string;
  keys: string;
}

export interface WorkspaceApp {
  name: string;
  path: string;
  exe_path?: string;
  icon?: string;
  x: number;
  y: number;
  width: number;
  height: number;
  urls?: string[];
}

export interface WorkspaceProfile {
  id: string;
  name: string;
  gap: number;
  apps: WorkspaceApp[];
}
