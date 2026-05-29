use serde::{Deserialize, Serialize};

use crate::processes::RunningProcess;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub general: GeneralPreferences,
    pub clipboard: ClipboardPreferences,
    pub window_manager: WindowManagerPreferences,
    pub power: PowerPreferences,
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            general: GeneralPreferences::default(),
            clipboard: ClipboardPreferences::default(),
            window_manager: WindowManagerPreferences::default(),
            power: PowerPreferences::default(),
            shortcuts: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralPreferences {
    pub launch_at_startup: bool,
    pub show_menu_bar: bool,
    pub language: String,
    pub theme: Theme,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            launch_at_startup: true,
            show_menu_bar: true,
            language: "en".to_string(),
            theme: Theme::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPreferences {
    pub enabled: bool,
    pub history_limit: i32,
    pub ignore_passwords: bool,
    pub ignored_apps: Vec<RunningProcess>,
}

impl Default for ClipboardPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            history_limit: 25,
            ignore_passwords: true,
            ignored_apps: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowManagerPreferences {
    pub snap_to_edges: bool,
    pub remember_position: bool,
    pub hide_on_focus_loss: bool,
    pub opacity: u8,
}

impl Default for WindowManagerPreferences {
    fn default() -> Self {
        Self {
            snap_to_edges: true,
            remember_position: true,
            hide_on_focus_loss: true,
            opacity: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPreferences {
    pub reduce_cpu_when_idle: bool,
    pub disable_animations_on_battery: bool,
    pub idle_timeout_seconds: u32,
}

impl Default for PowerPreferences {
    fn default() -> Self {
        Self {
            reduce_cpu_when_idle: true,
            disable_animations_on_battery: false,
            idle_timeout_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub action: String,
    pub keys: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}
