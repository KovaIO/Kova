use rusqlite::Result;
use std::sync::{Arc, Mutex};

use super::{
    limits::limits_for_tier,
    models::{LicenseInfo, LicenseTier},
    storage::LicenseStorage,
};

#[derive(Clone)]
pub struct LicenseService {
    storage: Arc<Mutex<LicenseStorage>>,
}

impl LicenseService {
    pub fn new(storage: LicenseStorage) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }

    pub fn get_info(&self) -> Result<LicenseInfo> {
        let tier = self.tier()?;
        Ok(LicenseInfo {
            limits: limits_for_tier(&tier),
            tier,
        })
    }

    pub fn tier(&self) -> Result<LicenseTier> {
        let storage = self.storage.lock().unwrap();
        storage.load_tier()
    }

    pub fn set_tier(&self, tier: LicenseTier) -> Result<LicenseInfo> {
        {
            let storage = self.storage.lock().unwrap();
            storage.save_tier(&tier)?;
        }
        Ok(LicenseInfo {
            limits: limits_for_tier(&tier),
            tier,
        })
    }
}
