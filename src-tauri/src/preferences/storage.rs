use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

use crate::{
    apps::get_process_icon,
    apps::InstalledApp,
    preferences::{
        AppearancePreferences, ClipboardPreferences, GeneralPreferences, PowerPreferences,
        Preferences, Shortcut,
    },
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
        preferences.appearance = self.load_appearance_preferences()?;
        preferences.power = self.load_power_preferences()?;
        preferences.shortcuts = self.load_shortcuts()?;
        Ok(preferences)
    }

    pub fn load_general_preferences(&self) -> Result<GeneralPreferences> {
        let mut stmt = self.conn.prepare(
            "SELECT launch_at_startup, show_menu_bar, language, monitor_dim FROM general_preferences LIMIT 1",
        )?;
        let prefs = stmt.query_row([], |row| {
            Ok(GeneralPreferences {
                launch_at_startup: row.get(0)?,
                show_menu_bar: row.get(1)?,
                language: row.get(2)?,
                monitor_dim: row.get(3)?,
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

    pub fn load_appearance_preferences(&self) -> Result<AppearancePreferences> {
        let mut stmt = self.conn.prepare(
            "SELECT accent_color, window_density, metric_card_style
             FROM appearance_preferences
             LIMIT 1",
        )?;

        stmt.query_row([], |row| {
            Ok(AppearancePreferences {
                accent_color: row.get(0)?,
                window_density: row.get(1)?,
                metric_card_style: row.get(2)?,
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

    fn load_ignored_apps(&self) -> Result<Vec<InstalledApp>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, path, exe_path FROM ignored_apps")?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            let exe_path: Option<String> = row.get(2)?;
            let icon = get_process_icon(&name, Some(&path));

            Ok(InstalledApp {
                name,
                path,
                exe_path,
                icon,
            })
        })?;
        let mut apps = vec![];
        for row in rows {
            apps.push(row?);
        }
        Ok(apps)
    }

    pub fn load_shortcuts(&self) -> Result<Vec<Shortcut>> {
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
        self.conn.execute(
            "UPDATE general_preferences SET launch_at_startup = ?1, show_menu_bar = ?2, language = ?3, monitor_dim = ?4",
            params![ prefs.launch_at_startup, prefs.show_menu_bar, prefs.language, prefs.monitor_dim ],)?;
        Ok(())
    }

    pub fn save_clipboard_preferences(&self, prefs: &ClipboardPreferences) -> Result<()> {
        self.conn.execute(
            "UPDATE clipboard_preferences SET enabled = ?1, history_limit = ?2, ignore_passwords = ?3",
            params![ prefs.enabled, prefs.history_limit, prefs.ignore_passwords ], )?;
        self.conn.execute("DELETE FROM ignored_apps", [])?;
        for app in &prefs.ignored_apps {
            self.conn.execute(
                "INSERT INTO ignored_apps ( name, path ) VALUES (?1, ?2)",
                params![app.name, app.path],
            )?;
        }
        Ok(())
    }

    pub fn save_shortcuts(&self, shortcuts: &[Shortcut]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM shortcuts", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO shortcuts (action, keys)
                 VALUES (?1, ?2)",
            )?;

            for shortcut in shortcuts {
                stmt.execute(params![shortcut.action, shortcut.keys,])?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    pub fn save_appearance_preferences(&self, prefs: &AppearancePreferences) -> Result<()> {
        self.conn.execute(
            "
            UPDATE appearance_preferences
            SET accent_color = ?1, window_density = ?2, metric_card_style = ?3
            ",
            params![prefs.accent_color, prefs.window_density, prefs.metric_card_style],
        )?;

        Ok(())
    }
}
