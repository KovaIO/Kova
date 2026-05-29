use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

use crate::{
    preferences::{
        ClipboardPreferences, GeneralPreferences, PowerPreferences, Preferences, Shortcut, Theme,
        WindowManagerPreferences,
    },
    processes::RunningProcess,
};

pub struct PreferencesStorage {
    conn: Connection,
}

impl PreferencesStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn load_preferences(&self) -> Result<Preferences> {
        let mut preferences = Preferences::default();
        preferences.general = self.load_general_preferences()?;
        preferences.clipboard = self.load_clipboard_preferences()?;
        preferences.window_manager = self.load_window_manager_preferences()?;
        preferences.power = self.load_power_preferences()?;
        preferences.shortcuts = self.load_shortcuts()?;
        Ok(preferences)
    }

    fn load_general_preferences(&self) -> Result<GeneralPreferences> {
        let mut stmt = self.conn.prepare("SELECT launch_at_startup, show_menu_bar, language, theme FROM general_preferences LIMIT 1",)?;
        let prefs = stmt.query_row([], |row| {
            let theme_str: String = row.get(3)?;
            let theme = match theme_str.as_str() {
                "dark" => Theme::Dark,
                "light" => Theme::Light,
                _ => Theme::System,
            };
            Ok(GeneralPreferences {
                launch_at_startup: row.get(0)?,
                show_menu_bar: row.get(1)?,
                language: row.get(2)?,
                theme,
            })
        })?;
        Ok(prefs)
    }

    fn load_clipboard_preferences(&self) -> Result<ClipboardPreferences> {
        let mut stmt = self.conn.prepare(
            "SELECT enabled, history_limit, ignore_passwords FROM clipboard_preferences LIMIT 1",
        )?;
        let mut prefs: ClipboardPreferences = stmt.query_row([], |row| {
            Ok(ClipboardPreferences {
                enabled: row.get(0)?,
                history_limit: row.get(1)?,
                ignore_passwords: row.get(2)?,
                ignored_apps: vec![],
            })
        })?;
        prefs.ignored_apps = self.load_ignored_apps()?;
        Ok(prefs)
    }

    fn load_window_manager_preferences(&self) -> Result<WindowManagerPreferences> {
        let mut stmt = self.conn.prepare("SELECT snap_to_edges, remember_position, hide_on_focus_loss, opacity FROM window_manager_preferences LIMIT 1",)?;
        stmt.query_row([], |row| {
            Ok(WindowManagerPreferences {
                snap_to_edges: row.get(0)?,
                remember_position: row.get(1)?,
                hide_on_focus_loss: row.get(2)?,
                opacity: row.get(3)?,
            })
        })
    }

    fn load_power_preferences(&self) -> Result<PowerPreferences> {
        let mut stmt = self.conn.prepare("SELECT reduce_cpu_when_idle, disable_animations_on_battery, idle_timeout_seconds FROM power_preferences LIMIT 1", )?;
        stmt.query_row([], |row| {
            Ok(PowerPreferences {
                reduce_cpu_when_idle: row.get(0)?,
                disable_animations_on_battery: row.get(1)?,
                idle_timeout_seconds: row.get(2)?,
            })
        })
    }

    fn load_ignored_apps(&self) -> Result<Vec<RunningProcess>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, path, icon FROM ignored_apps")?;
        let rows = stmt.query_map([], |row| {
            Ok(RunningProcess {
                name: row.get(0)?,
                path: row.get(1)?,
                icon: row.get(2)?,
            })
        })?;
        let mut apps = vec![];
        for row in rows {
            apps.push(row?);
        }
        Ok(apps)
    }

    fn load_shortcuts(&self) -> Result<Vec<Shortcut>> {
        let mut stmt = self.conn.prepare("SELECT action, keys FROM shortcuts")?;
        let rows = stmt.query_map([], |row| {
            Ok(Shortcut {
                action: row.get(0)?,
                keys: row.get(1)?,
            })
        })?;
        let mut shortcuts = vec![];
        for row in rows {
            shortcuts.push(row?);
        }
        Ok(shortcuts)
    }

    pub fn save_general_preferences(&self, prefs: &GeneralPreferences) -> Result<()> {
        let theme = match prefs.theme {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "system",
        };
        self.conn.execute(
            "UPDATE general_preferences SET launch_at_startup = ?1, show_menu_bar = ?2, language = ?3, theme = ?4",
            params![ prefs.launch_at_startup, prefs.show_menu_bar, prefs.language, theme ],)?;
        Ok(())
    }

    pub fn save_clipboard_preferences(&self, prefs: &ClipboardPreferences) -> Result<()> {
        self.conn.execute(
            "UPDATE clipboard_preferences SET enabled = ?1, history_limit = ?2, ignore_passwords = ?3",
            params![ prefs.enabled, prefs.history_limit, prefs.ignore_passwords ], )?;
        self.conn.execute("DELETE FROM ignored_apps", [])?;
        for app in &prefs.ignored_apps {
            self.conn.execute(
                "INSERT INTO ignored_apps ( name, path, icon ) VALUES (?1, ?2, ?3)",
                params![app.name, app.path, app.icon],
            )?;
        }
        Ok(())
    }

    pub fn save_window_manager_preferences(&self, prefs: &WindowManagerPreferences) -> Result<()> {
        self.conn.execute(
            "
            UPDATE window_manager_preferences
            SET
                snap_to_edges = ?1,
                remember_position = ?2,
                hide_on_focus_loss = ?3,
                opacity = ?4
            ",
            params![
                prefs.snap_to_edges,
                prefs.remember_position,
                prefs.hide_on_focus_loss,
                prefs.opacity
            ],
        )?;

        Ok(())
    }
}
