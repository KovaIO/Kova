use rusqlite::Result;
use std::sync::{Arc, Mutex};

use crate::license::{
    client::LicenseClient,
    models::{ActivateLicenseRequest, VerifyLicenseRequest},
};

use super::{
    limits::limits_for_tier,
    models::{LicenseInfo, LicenseTier},
    storage::LicenseStorage,
};

#[derive(Clone)]
pub struct LicenseService {
    storage: Arc<Mutex<LicenseStorage>>,
    client: Arc<LicenseClient>,
}

const VERIFY_INTERVAL_SECS: i64 = 60 * 60 * 24;

impl LicenseService {
    pub fn new(storage: LicenseStorage, client: LicenseClient) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
            client: Arc::new(client),
        }
    }

    pub fn get_info(&self) -> Result<LicenseInfo> {
        let storage = self.storage.lock().unwrap();

        let license = storage.load_license()?;

        Ok(LicenseInfo {
            device_id: license.device_id,
            email: license.email,
            activated_at: license.activated_at,
            last_verified_at: license.last_verified_at,
            limits: limits_for_tier(&license.tier),
            tier: license.tier,
        })
    }

    pub async fn activate_license(
        &self,
        email: String,
        device_name: String,
        platform: String,
    ) -> Result<LicenseInfo, String> {
        let device_id = {
            let storage = self.storage.lock().unwrap();
            storage.device_id().map_err(|e| e.to_string())?
        };

        let response = self
            .client
            .activate(ActivateLicenseRequest {
                email: email.clone(),
                device_id,
                device_name,
                platform,
            })
            .await?;

        let tier = if response.valid {
            LicenseTier::Pro
        } else {
            LicenseTier::Free
        };

        {
            let storage = self.storage.lock().unwrap();

            storage.activate(&email, &tier).map_err(|e| e.to_string())?;
        }

        self.get_info().map_err(|e| e.to_string())
    }

    pub async fn verify_license(&self) -> Result<LicenseInfo, String> {
        let license = {
            let storage = self.storage.lock().unwrap();

            storage.load_license().map_err(|e| e.to_string())?
        };

        let email = match &license.email {
            Some(email) => email.clone(),
            None => {
                return self.get_info().map_err(|e| e.to_string());
            }
        };

        let response = self
            .client
            .verify(VerifyLicenseRequest {
                email,
                device_id: license.device_id,
            })
            .await?;

        let tier = if response.valid {
            LicenseTier::Pro
        } else {
            LicenseTier::Free
        };

        {
            let storage = self.storage.lock().unwrap();

            storage
                .update_verification(&tier)
                .map_err(|e| e.to_string())?;
        }

        self.get_info().map_err(|e| e.to_string())
    }

    pub fn tier(&self) -> Result<LicenseTier> {
        let storage = self.storage.lock().unwrap();
        storage.load_tier()
    }

    pub fn should_verify(&self) -> Result<bool> {
        let storage = self.storage.lock().unwrap();

        let license = storage.load_license()?;

        if license.email.is_none() {
            return Ok(false);
        }

        match license.last_verified_at {
            None => Ok(true),

            Some(ts) => {
                let now = chrono::Utc::now().timestamp();

                Ok(now - ts >= VERIFY_INTERVAL_SECS)
            }
        }
    }

    pub async fn verify_if_needed(&self) -> Result<Option<LicenseInfo>, String> {
        let should_verify = self.should_verify().map_err(|e| e.to_string())?;

        if !should_verify {
            return Ok(None);
        }

        let info = self.verify_license().await?;

        Ok(Some(info))
    }

    pub async fn get_portal_url(&self) -> Result<String, String> {
        let email = {
            let storage = self.storage.lock().unwrap();
            let license = storage.load_license().map_err(|e| e.to_string())?;
            license
                .email
                .ok_or_else(|| "no email on file".to_string())?
        };

        self.client.get_portal_url(&email).await
    }
}
