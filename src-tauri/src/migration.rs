use crate::preferences::Preferences;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS general_preferences (
            launch_at_startup INTEGER NOT NULL DEFAULT 0,
            show_menu_bar INTEGER NOT NULL DEFAULT 1,
            language TEXT NOT NULL DEFAULT 'en',
            monitor_dim INTEGER NOT NULL DEFAULT 90
        );

        CREATE TABLE IF NOT EXISTS clipboard_preferences (
            enabled INTEGER NOT NULL DEFAULT 1,
            history_limit INTEGER NOT NULL DEFAULT 25,
            ignore_passwords INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS appearance_preferences (
            accent_color TEXT NOT NULL DEFAULT 'purple',
            window_density TEXT NOT NULL DEFAULT 'normal',
            metric_card_style TEXT NOT NULL DEFAULT 'block'
        );

        CREATE TABLE IF NOT EXISTS power_preferences (
            reduce_cpu_when_idle INTEGER NOT NULL DEFAULT 1,
            disable_animations_on_battery INTEGER NOT NULL DEFAULT 0,
            idle_timeout_seconds INTEGER NOT NULL DEFAULT 300
        );

        CREATE TABLE IF NOT EXISTS ignored_apps (
            path TEXT PRIMARY KEY,
            exe_path TEXT,
            name TEXT NOT NULL,
            icon TEXT
        );

        CREATE TABLE IF NOT EXISTS shortcuts (
            action TEXT PRIMARY KEY,
            keys TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS clipboard_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type TEXT NOT NULL,
            text_content TEXT,
            image_path TEXT,
            source_app TEXT,
            source_app_path TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workspace_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            gap INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS workspace_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,

            name TEXT NOT NULL,
            path TEXT NOT NULL,
            exe_path TEXT,
            icon TEXT,

            x REAL NOT NULL DEFAULT 0.0,
            y REAL NOT NULL DEFAULT 0.0,
            width REAL NOT NULL DEFAULT 0.5,
            height REAL NOT NULL DEFAULT 0.5
        );

        CREATE TABLE IF NOT EXISTS license (
            device_id TEXT NOT NULL,
            email TEXT,
            tier TEXT NOT NULL DEFAULT 'free',
            activated_at INTEGER,
            last_verified_at INTEGER
        );
        ",
    )?;

    seed_defaults(conn)?;

    Ok(())
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    let prefs = Preferences::default();
    let g = &prefs.general;
    let c = &prefs.clipboard;
    let p = &prefs.power;
    let a = &prefs.appearance;

    conn.execute(
        "INSERT INTO general_preferences (launch_at_startup, show_menu_bar, language, monitor_dim)
         SELECT ?1, ?2, ?3, ?4
         WHERE NOT EXISTS (SELECT 1 FROM general_preferences)",
        params![
            g.launch_at_startup as i32,
            g.show_menu_bar as i32,
            g.language,
            g.monitor_dim,
        ],
    )?;

    conn.execute(
        "INSERT INTO clipboard_preferences (enabled, history_limit, ignore_passwords)
         SELECT ?1, ?2, ?3
         WHERE NOT EXISTS (SELECT 1 FROM clipboard_preferences)",
        params![c.enabled as i32, c.history_limit, c.ignore_passwords as i32,],
    )?;

    conn.execute(
        "
            INSERT INTO appearance_preferences (accent_color, window_density, metric_card_style)
            SELECT ?1, ?2, ?3
            WHERE NOT EXISTS (
                SELECT 1 FROM appearance_preferences
            )
            ",
        params![a.accent_color, a.window_density, a.metric_card_style],
    )?;

    conn.execute(
        "INSERT INTO power_preferences (reduce_cpu_when_idle, disable_animations_on_battery, idle_timeout_seconds)
         SELECT ?1, ?2, ?3
         WHERE NOT EXISTS (SELECT 1 FROM power_preferences)",
        params![
            p.reduce_cpu_when_idle as i32,
            p.disable_animations_on_battery as i32,
            p.idle_timeout_seconds,
        ],
    )?;

    for shortcut in &prefs.shortcuts {
        conn.execute(
            "INSERT OR IGNORE INTO shortcuts (action, keys) VALUES (?1, ?2)",
            rusqlite::params![shortcut.action, shortcut.keys],
        )?;
    }

    let device_id = Uuid::new_v4().to_string();

    conn.execute(
        "
        INSERT INTO license (device_id, tier)
        SELECT ?1, 'free'
        WHERE NOT EXISTS (
            SELECT 1 FROM license
        )
        ",
        params![device_id],
    )?;

    Ok(())
}
