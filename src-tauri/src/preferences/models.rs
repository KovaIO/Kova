use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

use crate::apps::InstalledApp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub general: GeneralPreferences,
    pub clipboard: ClipboardPreferences,
    pub appearance: AppearancePreferences,
    pub power: PowerPreferences,
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            general: GeneralPreferences::default(),
            clipboard: ClipboardPreferences::default(),
            appearance: AppearancePreferences::default(),
            power: PowerPreferences::default(),
            shortcuts: vec![
                Shortcut {
                    action: ShortcutAction::OpenClipboardHistory,
                    keys: "Ctrl + Alt + Space".to_string(),
                },
                Shortcut {
                    action: ShortcutAction::ApplyWorkspace,
                    keys: "Alt + Shift + W".to_string(),
                },
                Shortcut {
                    action: ShortcutAction::OpenMonitor,
                    keys: "Alt + Shift + M".to_string(),
                },
                Shortcut {
                    action: ShortcutAction::OpenMenubarPopover,
                    keys: "Alt + Shift + P".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralPreferences {
    pub launch_at_startup: bool,
    pub show_menu_bar: bool,
    pub language: String,
    pub monitor_dim: u8,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            launch_at_startup: true,
            show_menu_bar: true,
            language: "en".to_string(),
            monitor_dim: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardPreferences {
    pub enabled: bool,
    pub history_limit: i32,
    pub ignore_passwords: bool,
    pub ignored_apps: Vec<InstalledApp>,
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
pub struct AppearancePreferences {
    pub accent_color: AccentColor,
    pub window_density: WindowDensity,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            accent_color: AccentColor::Purple,
            window_density: WindowDensity::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccentColor {
    Blue,
    Purple,
    Green,
    Orange,
    White,
}

impl FromSql for AccentColor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            "green" => Ok(Self::Green),
            "orange" => Ok(Self::Orange),
            "white" => Ok(Self::White),
            _ => Err(FromSqlError::Other(
                format!("Unknown shortcut action: {}", value.as_str()?).into(),
            )),
        }
    }
}

impl ToSql for AccentColor {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            Self::Blue => "blue".into(),
            Self::Purple => "purple".into(),
            Self::Green => "green".into(),
            Self::Orange => "orange".into(),
            Self::White => "white".into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WindowDensity {
    Normal,
    Wide,
}

impl FromSql for WindowDensity {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "normal" => Ok(Self::Normal),
            "wide" => Ok(Self::Wide),
            _ => Err(FromSqlError::Other(
                format!("Unknown shortcut action: {}", value.as_str()?).into(),
            )),
        }
    }
}

impl ToSql for WindowDensity {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            Self::Normal => "normal".into(),
            Self::Wide => "wide".into(),
        })
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
    pub action: ShortcutAction,
    pub keys: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortcutAction {
    ApplyWorkspace,

    OpenWindowSwitcher,
    PreviousWindow,
    SearchWindowSwitcher,
    ExpandTabs,
    CollapseTabs,

    OpenClipboardHistory,
    SearchClipboardHistory,

    OpenMonitor,
    OpenMenubarPopover,
}

impl FromSql for ShortcutAction {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "apply_workspace" => Ok(Self::ApplyWorkspace),

            "open_window_switcher" => Ok(Self::OpenWindowSwitcher),
            "previous_window" => Ok(Self::PreviousWindow),
            "search_window_switcher" => Ok(Self::SearchWindowSwitcher),
            "expand_tabs" => Ok(Self::ExpandTabs),
            "collapse_tabs" => Ok(Self::CollapseTabs),

            "open_clipboard_history" => Ok(Self::OpenClipboardHistory),
            "search_clipboard_history" => Ok(Self::SearchClipboardHistory),

            "open_monitor" => Ok(Self::OpenMonitor),
            "open_menubar_popover" => Ok(Self::OpenMenubarPopover),

            _ => Err(FromSqlError::Other(
                format!("Unknown shortcut action: {}", value.as_str()?).into(),
            )),
        }
    }
}

impl ToSql for ShortcutAction {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            Self::ApplyWorkspace => "apply_workspace".into(),

            Self::OpenWindowSwitcher => "open_window_switcher".into(),
            Self::PreviousWindow => "previous_window".into(),
            Self::SearchWindowSwitcher => "search_window_switcher".into(),
            Self::ExpandTabs => "expand_tabs".into(),
            Self::CollapseTabs => "collapse_tabs".into(),

            Self::OpenClipboardHistory => "open_clipboard_history".into(),
            Self::SearchClipboardHistory => "search_clipboard_history".into(),

            Self::OpenMonitor => "open_monitor".into(),
            Self::OpenMenubarPopover => "open_menubar_popover".into(),
        })
    }
}
