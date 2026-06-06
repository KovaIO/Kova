use rusqlite::{Connection, Result};
use std::path::PathBuf;

use super::models::LicenseTier;

pub struct LicenseStorage {
    conn: Connection,
}

impl LicenseStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn load_tier(&self) -> Result<LicenseTier> {
        let mut stmt = self.conn.prepare("SELECT tier FROM license LIMIT 1")?;
        let tier_str: String = stmt.query_row([], |row| row.get(0))?;
        Ok(LicenseTier::from_str(&tier_str))
    }

    pub fn save_tier(&self, tier: &LicenseTier) -> Result<()> {
        self.conn.execute(
            "UPDATE license SET tier = ?1",
            rusqlite::params![tier.as_str()],
        )?;
        Ok(())
    }
}
