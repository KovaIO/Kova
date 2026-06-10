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

/// Whether the user can actually delete this target, or only inspect its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupAction {
    /// User can delete this target.
    Delete,
    /// Report size only — never delete. For destructive targets like Docker/WSL VMs.
    AnalyzeOnly,
}

/// Determines how a scan target is located and deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanTargetKind {
    /// A single directory (e.g. npm cache, Chrome cache).
    Directory,
    /// A single file (e.g. MEMORY.DMP, IconCache.db, VHDX files).
    File,
    /// Glob pattern matching files within a parent directory.
    /// The `parent` is the directory, `pattern` is the glob.
    /// Example: parent=`C:\Windows\Explorer`, pattern=`thumbcache*.db`
    Pattern { parent: String, pattern: String },
    /// A virtual/OS-level entity that requires special APIs.
    /// Examples: Recycle Bin.
    Virtual { virtual_type: VirtualTarget },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualTarget {
    /// Windows Recycle Bin — cleared via SHEmptyRecycleBinW.
    RecycleBin,
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
    /// Whether deletion requires a special API (e.g. Recycle Bin).
    #[serde(default)]
    pub requires_virtual_delete: bool,
    /// Process names that must not be running for safe deletion.
    #[serde(default)]
    pub locks_process: Vec<String>,
    /// For pattern targets: the parent directory.
    #[serde(default)]
    pub pattern_parent: Option<String>,
    /// For pattern targets: the glob pattern (e.g. "thumbcache_*.db").
    #[serde(default)]
    pub pattern_glob: Option<String>,
    /// For virtual targets: "RecycleBin".
    #[serde(default)]
    pub virtual_type_str: Option<String>,
    /// Whether this item can be deleted or is analyze-only.
    #[serde(default = "default_cleanup_action")]
    pub action: CleanupAction,
}

fn default_cleanup_action() -> CleanupAction {
    CleanupAction::Delete
}

impl DiskScanItem {
    pub fn virtual_type(&self) -> Option<VirtualTarget> {
        match self.virtual_type_str.as_deref() {
            Some("RecycleBin") => Some(VirtualTarget::RecycleBin),
            _ => None,
        }
    }

    pub fn scan_target_kind(&self) -> ScanTargetKind {
        if self.requires_virtual_delete {
            let vt = self.virtual_type().unwrap_or(VirtualTarget::RecycleBin);
            return ScanTargetKind::Virtual { virtual_type: vt };
        }
        if let (Some(parent), Some(pattern)) = (&self.pattern_parent, &self.pattern_glob) {
            return ScanTargetKind::Pattern {
                parent: parent.clone(),
                pattern: pattern.clone(),
            };
        }
        if self.is_dir {
            ScanTargetKind::Directory
        } else {
            ScanTargetKind::File
        }
    }
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
    pub kind: ScanTargetKind,
    pub safety: DiskSafety,
    pub safety_reason: String,
    pub locks_process: Vec<String>,
    pub action: CleanupAction,
}
