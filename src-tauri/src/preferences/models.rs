use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

use crate::clipboard::models::InstalledApp;

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
            shortcuts: vec![Shortcut {
                action: ShortcutAction::OpenClipboardHistory,
                keys: "Ctrl + alt + Space".to_string(),
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralPreferences {
    pub launch_at_startup: bool,
    pub show_menu_bar: bool,
    pub language: String,
    pub theme: Theme,
    pub monitor_dim: u8,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            launch_at_startup: true,
            show_menu_bar: true,
            language: "en".to_string(),
            theme: Theme::System,
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
pub struct WindowManagerPreferences {
    pub enabled: bool,
    pub auto_layout: bool,
    pub window_switcher: bool,
}

impl Default for WindowManagerPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_layout: true,
            window_switcher: true,
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
    pub action: ShortcutAction,
    pub keys: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl ToSql for Theme {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            Self::Dark => "dark".into(),
            Self::Light => "light".into(),
            Self::System => "system".into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortcutAction {
    MoveWindowToNextScreen,
    MatchWithAnotherWindow,
    AutoLayoutWindows,
    CenterWindow,
    MakeWindow16By9,

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
            "move_window_to_next_screen" => Ok(Self::MoveWindowToNextScreen),
            "match_with_another_window" => Ok(Self::MatchWithAnotherWindow),
            "auto_layout_windows" => Ok(Self::AutoLayoutWindows),
            "center_window" => Ok(Self::CenterWindow),
            "make_window_16_by_9" => Ok(Self::MakeWindow16By9),

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
            Self::MoveWindowToNextScreen => "move_window_to_next_screen".into(),
            Self::MatchWithAnotherWindow => "match_with_another_window".into(),
            Self::AutoLayoutWindows => "auto_layout_windows".into(),
            Self::CenterWindow => "center_window".into(),
            Self::MakeWindow16By9 => "make_window_16_by_9".into(),

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
