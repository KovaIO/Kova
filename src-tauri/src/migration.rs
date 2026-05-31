use rusqlite::{params, Connection, Result};

use crate::preferences::Preferences;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS general_preferences (
            launch_at_startup INTEGER NOT NULL DEFAULT 0,
            show_menu_bar INTEGER NOT NULL DEFAULT 1,
            language TEXT NOT NULL DEFAULT 'en',
            theme TEXT NOT NULL DEFAULT 'system',
            monitor_dim INTEGER NOT NULL DEFAULT 90
        );

        CREATE TABLE IF NOT EXISTS clipboard_preferences (
            enabled INTEGER NOT NULL DEFAULT 1,
            history_limit INTEGER NOT NULL DEFAULT 25,
            ignore_passwords INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS window_manager_preferences (
            enabled INTEGER NOT NULL DEFAULT 1,
            auto_layout INTEGER NOT NULL DEFAULT 1,
            window_switcher INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS power_preferences (
            reduce_cpu_when_idle INTEGER NOT NULL DEFAULT 1,
            disable_animations_on_battery INTEGER NOT NULL DEFAULT 0,
            idle_timeout_seconds INTEGER NOT NULL DEFAULT 300
        );

        CREATE TABLE IF NOT EXISTS ignored_apps (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT
        );

        CREATE TABLE IF NOT EXISTS shortcuts (
            action TEXT PRIMARY KEY,
            keys TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS license (
            tier TEXT NOT NULL DEFAULT 'free'
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
    let w = &prefs.window_manager;
    let p = &prefs.power;

    conn.execute(
        "INSERT INTO general_preferences (launch_at_startup, show_menu_bar, language, theme, monitor_dim)
         SELECT ?1, ?2, ?3, ?4, ?5
         WHERE NOT EXISTS (SELECT 1 FROM general_preferences)",
        params![
            g.launch_at_startup as i32,
            g.show_menu_bar as i32,
            g.language,
            g.theme,
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
        "INSERT INTO window_manager_preferences (enabled, auto_layout, window_switcher)
         SELECT ?1, ?2, ?3
         WHERE NOT EXISTS (SELECT 1 FROM window_manager_preferences)",
        params![
            w.enabled as i32,
            w.auto_layout as i32,
            w.window_switcher as i32,
        ],
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

    conn.execute(
        "
        INSERT INTO license (tier)
        SELECT 'free'
        WHERE NOT EXISTS (
            SELECT 1 FROM license
        )
        ",
        [],
    )?;

    Ok(())
}
