use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use sysinfo::Disks;
use tauri::{AppHandle, Emitter};

use crate::disk::{
    models::{
        CleanupAction, DiskCategory, DiskCategoryGroup, DiskItemChild, DiskItemDetail,
        DiskScanItem, DiskScanProgress, DiskScanResult, DiskSafety, DiskVolumeInfo, ScanPreview,
        ScanPreviewCategory, ScanTargetKind, VirtualTarget,
    },
    scanner::{
        dir_size, empty_recycle_bin, file_size, is_any_process_running, is_path_deletable,
        item_id, list_children, pattern_files, primary_mount_path, recycle_bin_size,
        scan_targets,
    },
    storage::DiskStorage,
};
use crate::metrics::disk::{find_disk_index, primary_disk_metrics};

pub struct DiskService {
    storage: Mutex<DiskStorage>,
    scanning: Mutex<bool>,
    allowed_roots: Mutex<Vec<String>>,
}

impl DiskService {
    pub fn new(storage: DiskStorage) -> Self {
        let roots = scan_targets().into_iter().map(|t| t.path).collect();

        Self {
            storage: Mutex::new(storage),
            scanning: Mutex::new(false),
            allowed_roots: Mutex::new(roots),
        }
    }

    pub fn volume_info(&self) -> DiskVolumeInfo {
        let mount = primary_mount_path();
        let mut disks = Disks::new_with_refreshed_list();
        let index = find_disk_index(&disks, Path::new(&mount));
        disks.refresh(false);
        let (used, total, percent) = primary_disk_metrics(&disks, index);
        let available = total.saturating_sub(used);

        let last_scan_at = self
            .storage
            .lock()
            .ok()
            .and_then(|s| s.last_scan_at());

        DiskVolumeInfo {
            mount_path: mount,
            label: volume_label(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            used_percent: percent,
            last_scan_at,
        }
    }

    pub fn scan_preview(&self) -> ScanPreview {
        let categories = [
            DiskCategory::System,
            DiskCategory::Browsers,
            DiskCategory::Development,
            DiskCategory::Applications,
            DiskCategory::Storage,
            DiskCategory::Other,
        ]
        .into_iter()
        .map(|category| ScanPreviewCategory {
            label: category.label().to_string(),
            description: category_description(category),
            category,
        })
        .collect();

        ScanPreview {
            mount_path: primary_mount_path(),
            categories,
        }
    }

    pub fn get_scan_result(&self) -> Option<DiskScanResult> {
        self.storage.lock().ok()?.load()
    }

    pub fn get_item_detail(&self, item_id: &str) -> Result<DiskItemDetail, String> {
        let result = self
            .get_scan_result()
            .ok_or_else(|| "No scan results available".to_string())?;

        let item = result
            .categories
            .iter()
            .flat_map(|group| group.items.iter())
            .find(|item| item.id == item_id)
            .cloned()
            .ok_or_else(|| "Item not found".to_string())?;

        let children = if item.requires_virtual_delete {
            // Virtual items don't have browsable children
            vec![]
        } else {
            list_children(Path::new(&item.path), 40)
                .into_iter()
                .map(|(name, path, size_bytes, is_dir)| DiskItemChild {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size_bytes,
                    is_dir,
                })
                .collect()
        };

        Ok(DiskItemDetail { item, children })
    }

    pub async fn delete_items(self: &Arc<Self>, ids: Vec<String>) -> Result<u64, String> {
        let this = Arc::clone(self);
        tokio::task::spawn_blocking(move || this.delete_items_blocking(&ids))
            .await
            .map_err(|e| e.to_string())?
    }

    fn delete_items_blocking(&self, ids: &[String]) -> Result<u64, String> {
        let result = self
            .get_scan_result()
            .ok_or_else(|| "No scan results available".to_string())?;

        let allowed = self
            .allowed_roots
            .lock()
            .map_err(|e| e.to_string())?
            .clone();

        let mut freed = 0u64;
        let mut deleted_ids = Vec::new();

        for id in ids {
            let Some(item) = find_item(&result, id) else {
                continue;
            };

            if item.safety == DiskSafety::Unsafe {
                continue;
            }

            // Analyze-only items (Docker, WSL) cannot be deleted
            if item.action == CleanupAction::AnalyzeOnly {
                continue;
            }

            // Check if a blocking process is running
            if is_any_process_running(&item.locks_process) {
                continue;
            }

            // Handle virtual targets (Recycle Bin)
            if item.requires_virtual_delete {
                match item.virtual_type() {
                    Some(VirtualTarget::RecycleBin) => {
                        match empty_recycle_bin() {
                            Ok(freed_bytes) => {
                                freed += freed_bytes;
                                deleted_ids.push(id.clone());
                            }
                            Err(_) => continue,
                        }
                    }
                    _ => continue,
                }
                continue;
            }

            // Check path is within allowed roots
            if !is_path_deletable(Path::new(&item.path), &allowed) {
                continue;
            }

            match delete_path_by_kind(&item.path, &item.scan_target_kind()) {
                Ok(()) => {
                    freed += item.size_bytes;
                    deleted_ids.push(id.clone());
                }
                Err(_) => continue,
            }
        }

        if deleted_ids.is_empty() {
            return Ok(0);
        }

        let mut updated = result;
        for group in &mut updated.categories {
            group.items.retain(|item| !deleted_ids.contains(&item.id));
            group.total_bytes = group.items.iter().map(|i| i.size_bytes).sum();
        }
        updated.categories.retain(|g| !g.items.is_empty());
        updated.total_reclaimable = updated
            .categories
            .iter()
            .flat_map(|g| g.items.iter())
            .map(|i| i.size_bytes)
            .sum();

        self.storage
            .lock()
            .map_err(|e| e.to_string())?
            .save(&updated)?;

        Ok(freed)
    }

    pub async fn run_scan(self: &Arc<Self>, app: AppHandle) -> Result<DiskScanResult, String> {
        {
            let mut scanning = self.scanning.lock().map_err(|e| e.to_string())?;
            if *scanning {
                return Err("Scan already in progress".into());
            }
            *scanning = true;
        }

        let service = Arc::clone(self);
        let result = tauri::async_runtime::spawn_blocking(move || service.scan_blocking(&app))
            .await
            .map_err(|e| e.to_string())?;

        {
            let mut scanning = self.scanning.lock().map_err(|e| e.to_string())?;
            *scanning = false;
        }

        result
    }

    fn scan_blocking(&self, app: &AppHandle) -> Result<DiskScanResult, String> {
        let targets = scan_targets();
        let total_targets = targets.len().max(1);
        let volume = self.volume_info();
        let mut groups: HashMap<DiskCategory, DiskCategoryGroup> = HashMap::new();

        for (index, target) in targets.iter().enumerate() {
            let progress = ((index + 1) as f32 / total_targets as f32 * 100.0) as u8;
            let _ = app.emit(
                "disk-scan-progress",
                DiskScanProgress {
                    progress,
                    message: format!("Scanning {}", target.name),
                    phase: "scanning".into(),
                },
            );

            // Compute size based on target kind
            let (size_bytes, item_count, is_dir) = match &target.kind {
                ScanTargetKind::Directory => {
                    let path = Path::new(&target.path);
                    if !path.exists() {
                        continue;
                    }
                    let (size, count) = dir_size(path, 4);
                    (size, count, true)
                }
                ScanTargetKind::File => {
                    let path = Path::new(&target.path);
                    if !path.exists() {
                        continue;
                    }
                    (file_size(path), 1, false)
                }
                ScanTargetKind::Pattern { parent, pattern } => {
                    let (size, count) = crate::disk::scanner::pattern_size(parent, pattern);
                    if size == 0 {
                        continue;
                    }
                    (size, count, false)
                }
                ScanTargetKind::Virtual { virtual_type } => {
                    let size = match virtual_type {
                        VirtualTarget::RecycleBin => recycle_bin_size(),
                    };
                    if size == 0 {
                        continue;
                    }
                    (size, 1, true)
                }
            };

            if size_bytes == 0 {
                continue;
            }

            let item = DiskScanItem {
                id: item_id(&target.path),
                name: target.name.clone(),
                path: target.path.clone(),
                size_bytes,
                category: target.category,
                is_dir,
                item_count,
                safety: target.safety,
                safety_reason: target.safety_reason.clone(),
                requires_virtual_delete: matches!(&target.kind, ScanTargetKind::Virtual { .. }),
                locks_process: target.locks_process.clone(),
                pattern_parent: match &target.kind {
                    ScanTargetKind::Pattern { parent, .. } => Some(parent.clone()),
                    _ => None,
                },
                pattern_glob: match &target.kind {
                    ScanTargetKind::Pattern { pattern, .. } => Some(pattern.clone()),
                    _ => None,
                },
                virtual_type_str: match &target.kind {
                    ScanTargetKind::Virtual { virtual_type } => Some(format!("{:?}", virtual_type)),
                    _ => None,
                },
                action: target.action,
            };

            let group = groups.entry(target.category).or_insert_with(|| DiskCategoryGroup {
                category: target.category,
                label: target.category.label().to_string(),
                total_bytes: 0,
                items: Vec::new(),
            });

            group.total_bytes += size_bytes;
            group.items.push(item);
        }

        let mut categories: Vec<DiskCategoryGroup> = groups.into_values().collect();
        for group in &mut categories {
            group.items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        }
        categories.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

        let total_reclaimable = categories
            .iter()
            .flat_map(|g| g.items.iter())
            .map(|i| i.size_bytes)
            .sum();

        let scanned_at = unix_now();
        let result = DiskScanResult {
            scanned_at,
            mount_path: volume.mount_path,
            total_bytes: volume.total_bytes,
            used_bytes: volume.used_bytes,
            available_bytes: volume.available_bytes,
            total_reclaimable,
            categories,
        };

        self.storage
            .lock()
            .map_err(|e| e.to_string())?
            .save(&result)?;

        let _ = app.emit(
            "disk-scan-progress",
            DiskScanProgress {
                progress: 100,
                message: "Scan complete".into(),
                phase: "complete".into(),
            },
        );

        Ok(result)
    }
}

fn find_item<'a>(result: &'a DiskScanResult, id: &str) -> Option<&'a DiskScanItem> {
    result
        .categories
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.id == id)
}

fn delete_path(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return std::fs::remove_file(path).map_err(|e| e.to_string());
    }

    // Try fast removal first
    if std::fs::remove_dir_all(path).is_ok() {
        return Ok(());
    }

    // Fallback: delete files one by one, skip locked ones
    delete_dir_contents(path)?;

    // Try removing the now-empty directory
    let _ = std::fs::remove_dir(path);
    Ok(())
}

/// Delete a path based on its ScanTargetKind.
fn delete_path_by_kind(path_str: &str, kind: &ScanTargetKind) -> Result<(), String> {
    match kind {
        ScanTargetKind::Directory => {
            let path = Path::new(path_str);
            if path.is_dir() {
                delete_path(path)
            } else {
                Ok(())
            }
        }
        ScanTargetKind::File => {
            let path = Path::new(path_str);
            if path.is_file() {
                std::fs::remove_file(path).map_err(|e| e.to_string())
            } else {
                Ok(())
            }
        }
        ScanTargetKind::Pattern { parent, pattern } => {
            let files = pattern_files(parent, pattern);
            let mut last_err = Ok(());
            for file in &files {
                if let Err(e) = std::fs::remove_file(file) {
                    last_err = Err(e.to_string());
                }
            }
            last_err
        }
        ScanTargetKind::Virtual { .. } => {
            // Virtual targets handled separately before reaching here
            Ok(())
        }
    }
}

/// Recursively deletes all files inside a directory, skipping locked ones.
/// After files are deleted, tries to remove empty subdirectories.
fn delete_dir_contents(dir: &Path) -> Result<(), String> {
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };

        let mut subdirs = Vec::new();

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                subdirs.push(entry_path);
            } else {
                // Try to delete file, skip if locked
                let _ = std::fs::remove_file(&entry_path);
            }
        }

        // Process subdirectories depth-first
        for sub in subdirs {
            stack.push(sub);
        }
    }

    Ok(())
}

fn volume_label() -> String {
    #[cfg(target_os = "windows")]
    {
        return std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        return "Macintosh HD".to_string();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "System".to_string()
    }
}

fn category_description(category: DiskCategory) -> String {
    match category {
        DiskCategory::System => "Temporary files, logs, and system caches".into(),
        DiskCategory::Browsers => "Browser caches and web data".into(),
        DiskCategory::Development => "npm, pip, cargo, Xcode, and dev tool caches".into(),
        DiskCategory::Applications => "App-specific cache folders".into(),
        DiskCategory::Storage => "Thumbnails, downloads, and large storage caches".into(),
        DiskCategory::Other => "Miscellaneous safe-to-review cache locations".into(),
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
