use rusqlite::{Connection, Result};
use std::path::PathBuf;

use crate::license::models::StoredLicense;

use super::models::LicenseTier;

pub struct LicenseStorage {
    conn: Connection,
}

impl LicenseStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn load_license(&self) -> Result<StoredLicense> {
        let mut stmt = self.conn.prepare(
            " SELECT device_id, email, tier, activated_at, last_verified_at
            FROM license
            LIMIT 1
            ",
        )?;

        stmt.query_row([], |row| {
            Ok(StoredLicense {
                device_id: row.get(0)?,
                email: row.get(1)?,
                tier: LicenseTier::from_str(&row.get::<_, String>(2)?),
                activated_at: row.get(3)?,
                last_verified_at: row.get(4)?,
            })
        })
    }

    pub fn activate(&self, email: &str, tier: &LicenseTier) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "
            UPDATE license
            SET
                email = ?1,
                tier = ?2,
                activated_at = ?3,
                last_verified_at = ?3
            ",
            rusqlite::params![email, tier.as_str(), now,],
        )?;

        Ok(())
    }

    pub fn update_verification(&self, tier: &LicenseTier) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "
            UPDATE license
            SET
                tier = ?1,
                last_verified_at = ?2
            ",
            rusqlite::params![tier.as_str(), now,],
        )?;

        Ok(())
    }

    pub fn device_id(&self) -> Result<String> {
        let mut stmt = self.conn.prepare("SELECT device_id FROM license LIMIT 1")?;

        stmt.query_row([], |row| row.get(0))
    }

    pub fn load_tier(&self) -> Result<LicenseTier> {
        let mut stmt = self.conn.prepare("SELECT tier FROM license LIMIT 1")?;
        let tier_str: String = stmt.query_row([], |row| row.get(0))?;
        Ok(LicenseTier::from_str(&tier_str))
    }
}
