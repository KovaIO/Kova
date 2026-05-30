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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub limits: LicenseLimits,
}

impl LicenseTier {
    pub fn from_str(value: &str) -> Self {
        match value {
            "pro" => Self::Pro,
            _ => Self::Free,
        }
    }
}
