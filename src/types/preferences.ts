export interface Preferences {
  general: GeneralPreferences;
  clipboard: ClipboardPreferences;
  window_manager: WindowManagerPreferences;
  power: PowerPreferences;
  shortcuts: Shortcut[];
}

export interface GeneralPreferences {
  launch_at_startup: boolean;
  show_menu_bar: boolean;
  language: string;
  theme: "dark" | "light" | "system";
}

export interface ClipboardPreferences {
  enabled: boolean;
  history_limit: number;
  ignore_passwords: boolean;
  ignored_apps: IgnoredApp[];
}

export interface WindowManagerPreferences {
  snap_to_edges: boolean;
  remember_position: boolean;
  hide_on_focus_loss: boolean;
  opacity: number;
}

export interface PowerPreferences {
  reduce_cpu_when_idle: boolean;
  disable_animations_on_battery: boolean;
  idle_timeout_seconds: number;
}

export interface IgnoredApp {
  name: string;
  path: string;
  icon?: string;
}

export interface Shortcut {
  action: string;
  keys: string;
}
