use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::models::DiskScanResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredScan {
    pub last_scan_at: Option<i64>,
    pub result: Option<DiskScanResult>,
}

pub struct DiskStorage {
    scan_file: PathBuf,
}

impl DiskStorage {
    pub fn new(app_dir: PathBuf) -> Self {
        let disk_dir = app_dir.join("disk");
        let _ = std::fs::create_dir_all(&disk_dir);
        Self {
            scan_file: disk_dir.join("last_scan.json"),
        }
    }

    pub fn load(&self) -> Option<DiskScanResult> {
        let raw = std::fs::read_to_string(&self.scan_file).ok()?;
        let stored: StoredScan = serde_json::from_str(&raw).ok()?;
        stored.result
    }

    pub fn last_scan_at(&self) -> Option<i64> {
        let raw = std::fs::read_to_string(&self.scan_file).ok()?;
        let stored: StoredScan = serde_json::from_str(&raw).ok()?;
        stored.last_scan_at
    }

    pub fn save(&self, result: &DiskScanResult) -> Result<(), String> {
        let stored = StoredScan {
            last_scan_at: Some(result.scanned_at),
            result: Some(result.clone()),
        };
        let json = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
        std::fs::write(&self.scan_file, json).map_err(|e| e.to_string())
    }
}
