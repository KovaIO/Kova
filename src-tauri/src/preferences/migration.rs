use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS general_preferences (
            launch_at_startup INTEGER NOT NULL DEFAULT 0,
            show_menu_bar INTEGER NOT NULL DEFAULT 1,
            language TEXT NOT NULL DEFAULT 'en',
            theme TEXT NOT NULL DEFAULT 'system'
        );

        CREATE TABLE IF NOT EXISTS clipboard_preferences (
            enabled INTEGER NOT NULL DEFAULT 1,
            history_limit INTEGER NOT NULL DEFAULT 25,
            ignore_passwords INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS window_manager_preferences (
            snap_to_edges INTEGER NOT NULL DEFAULT 1,
            remember_position INTEGER NOT NULL DEFAULT 1,
            hide_on_focus_loss INTEGER NOT NULL DEFAULT 1,
            opacity INTEGER NOT NULL DEFAULT 90
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
        ",
    )?;

    seed_defaults(conn)?;

    Ok(())
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        INSERT INTO general_preferences (
            launch_at_startup,
            show_menu_bar,
            language,
            theme
        )
        SELECT 0, 1, 'en', 'system'
        WHERE NOT EXISTS (
            SELECT 1 FROM general_preferences
        )
        ",
        [],
    )?;

    conn.execute(
        "
        INSERT INTO clipboard_preferences (
            enabled,
            history_limit,
            ignore_passwords
        )
        SELECT 1, 25, 1
        WHERE NOT EXISTS (
            SELECT 1 FROM clipboard_preferences
        )
        ",
        [],
    )?;

    conn.execute(
        "
        INSERT INTO window_manager_preferences (
            snap_to_edges,
            remember_position,
            hide_on_focus_loss,
            opacity
        )
        SELECT 1, 1, 1, 90
        WHERE NOT EXISTS (
            SELECT 1 FROM window_manager_preferences
        )
        ",
        [],
    )?;

    conn.execute(
        "
        INSERT INTO power_preferences (
            reduce_cpu_when_idle,
            disable_animations_on_battery,
            idle_timeout_seconds
        )
        SELECT 1, 0, 300
        WHERE NOT EXISTS (
            SELECT 1 FROM power_preferences
        )
        ",
        [],
    )?;

    Ok(())
}
