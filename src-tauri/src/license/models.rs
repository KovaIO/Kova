use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    Free,
    Pro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseLimits {
    pub clipboard_history_unlimited: bool,
    pub monitor_dimming: bool,
    pub disk_clean: bool,
    pub auto_layout: bool,
    pub window_switcher: bool,
    pub workspace_profiles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub limits: LicenseLimits,
    pub device_id: String,
    pub email: Option<String>,
    pub activated_at: Option<i64>,
    pub last_verified_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct StoredLicense {
    pub device_id: String,
    pub email: Option<String>,
    pub tier: LicenseTier,
    pub activated_at: Option<i64>,
    pub last_verified_at: Option<i64>,
}

impl LicenseTier {
    pub fn from_str(value: &str) -> Self {
        match value {
            "pro" => Self::Pro,
            _ => Self::Free,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Free => "free",
            _ => "pro",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ActivateLicenseRequest {
    pub email: String,
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyLicenseRequest {
    pub email: String,
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LicenseResponse {
    pub plan: String,
    pub status: String,
    pub device_limit: i32,
    pub active_devices: i32,
    pub valid: bool,
}
