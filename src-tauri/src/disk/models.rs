use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskCategory {
    System,
    Browsers,
    Development,
    Applications,
    Storage,
    Other,
}

impl DiskCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Browsers => "Browsers",
            Self::Development => "Development",
            Self::Applications => "Applications",
            Self::Storage => "Storage",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskSafety {
    Safe,
    Caution,
    Unsafe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskVolumeInfo {
    pub mount_path: String,
    pub label: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: u8,
    pub last_scan_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPreviewCategory {
    pub category: DiskCategory,
    pub label: String,
    pub description: String,
    pub target_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPreview {
    pub mount_path: String,
    pub categories: Vec<ScanPreviewCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScanProgress {
    pub progress: u8,
    pub message: String,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScanItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub category: DiskCategory,
    pub is_dir: bool,
    pub item_count: u32,
    pub safety: DiskSafety,
    pub safety_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCategoryGroup {
    pub category: DiskCategory,
    pub label: String,
    pub total_bytes: u64,
    pub items: Vec<DiskScanItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskScanResult {
    pub scanned_at: i64,
    pub mount_path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_reclaimable: u64,
    pub categories: Vec<DiskCategoryGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskItemChild {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskItemDetail {
    pub item: DiskScanItem,
    pub children: Vec<DiskItemChild>,
}

#[derive(Debug, Clone)]
pub struct ScanTarget {
    pub category: DiskCategory,
    pub name: String,
    pub path: String,
    pub safety: DiskSafety,
    pub safety_reason: String,
}
