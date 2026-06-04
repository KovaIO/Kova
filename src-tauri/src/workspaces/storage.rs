use crate::workspaces::{WorkspaceApp, WorkspaceProfile};
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct WorkspaceStorage {
    conn: Connection,
}

impl WorkspaceStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn load_profiles(&self) -> Result<Vec<WorkspaceProfile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM workspace_profiles ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut profiles = Vec::new();
        for row in rows {
            let (id, name) = row?;
            profiles.push(WorkspaceProfile {
                apps: self.load_apps(&id)?,
                id,
                name,
            });
        }
        Ok(profiles)
    }

    pub fn load_profile(&self, profile_id: &str) -> Result<Option<WorkspaceProfile>> {
        let result = self.conn.query_row(
            "SELECT id, name FROM workspace_profiles WHERE id = ?1",
            [profile_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match result {
            Ok((id, name)) => Ok(Some(WorkspaceProfile {
                apps: self.load_apps(&id)?,
                id,
                name,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn load_apps(&self, profile_id: &str) -> Result<Vec<WorkspaceApp>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, path, exe_path, icon, x, y, width, height
             FROM workspace_apps
             WHERE profile_id = ?1
             ORDER BY rowid",
        )?;
        let rows = stmt.query_map([profile_id], |row| {
            Ok(WorkspaceApp {
                name: row.get(0)?,
                path: row.get(1)?,
                exe_path: row.get(2)?,
                icon: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                width: row.get(6)?,
                height: row.get(7)?,
            })
        })?;

        rows.collect::<Result<Vec<_>>>()
    }

    pub fn save_profile(&self, profile: &WorkspaceProfile) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO workspace_profiles (id, name)
             VALUES (?1, ?2)",
            params![profile.id, profile.name],
        )?;

        tx.execute(
            "DELETE FROM workspace_apps WHERE profile_id = ?1",
            [profile.id.as_str()],
        )?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO workspace_apps
                    (profile_id, name, path, icon, x, y, width, height)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for app in &profile.apps {
                stmt.execute(params![
                    profile.id, app.name, app.path, app.icon, app.x, app.y, app.width, app.height,
                ])?;
            }
        }

        tx.commit()
    }

    pub fn delete_profile(&self, profile_id: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM workspace_apps WHERE profile_id = ?1",
            [profile_id],
        )?;
        tx.execute("DELETE FROM workspace_profiles WHERE id = ?1", [profile_id])?;
        tx.commit()
    }
}
