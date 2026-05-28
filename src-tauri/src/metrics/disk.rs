use std::path::Path;
use sysinfo::Disks;

pub fn find_disk_index(disks: &Disks, target: &Path) -> Option<usize> {
    disks.list().iter().position(|d| d.mount_point() == target)
}

pub fn primary_disk_metrics(disks: &Disks, index: Option<usize>) -> (u64, u64, u8) {
    let disk = index.and_then(|i| disks.list().get(i));
    match disk {
        Some(disk) => {
            let total = disk.total_space();
            let used = total.saturating_sub(disk.available_space());
            let percent = if total > 0 {
                ((used as f32 / total as f32) * 100.0).round() as u8
            } else {
                0
            };
            (used, total, percent)
        }
        None => (0, 0, 0),
    }
}

#[cfg(target_os = "windows")]
pub fn windows_system_drive() -> String {
    std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()) + "\\"
}
