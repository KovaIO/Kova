use std::path::{Path, PathBuf};

use crate::disk::models::{DiskCategory, DiskSafety, ScanTarget};

pub fn scan_targets() -> Vec<ScanTarget> {
    let mut targets = Vec::new();

    #[cfg(target_os = "windows")]
    windows_targets(&mut targets);

    #[cfg(target_os = "macos")]
    macos_targets(&mut targets);

    targets
}

pub fn item_id(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn dir_size(path: &Path, max_depth: u32) -> (u64, u32) {
    dir_size_inner(path, max_depth, 0)
}

fn dir_size_inner(path: &Path, max_depth: u32, depth: u32) -> (u64, u32) {
    if depth > max_depth {
        return (0, 0);
    }

    let mut total = 0u64;
    let mut count = 0u32;

    let Ok(entries) = std::fs::read_dir(path) else {
        return (0, 0);
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        count += 1;

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() {
            let (sub_size, sub_count) = dir_size_inner(&entry_path, max_depth, depth + 1);
            total += sub_size;
            count += sub_count;
        }
    }

    (total, count)
}

pub fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub fn list_children(path: &Path, limit: usize) -> Vec<(String, PathBuf, u64, bool)> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return vec![];
    };

    let mut items: Vec<(String, PathBuf, u64, bool)> = entries
        .flatten()
        .filter_map(|entry| {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok()?;
            let is_dir = meta.is_dir();
            let size = if is_dir {
                dir_size(&p, 1).0
            } else {
                meta.len()
            };
            Some((name, p, size, is_dir))
        })
        .collect();

    items.sort_by(|a, b| b.2.cmp(&a.2));
    items.truncate(limit);
    items
}

pub fn is_path_deletable(path: &Path, allowed_roots: &[String]) -> bool {
    let normalized = path_to_key(path);
    allowed_roots.iter().any(|root| {
        let root_key = path_to_key(Path::new(root));
        normalized.starts_with(&root_key)
    })
}

fn path_to_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

#[cfg(target_os = "windows")]
fn windows_targets(out: &mut Vec<ScanTarget>) {
    use std::env;

    let local = env::var("LOCALAPPDATA").unwrap_or_default();
    let appdata = env::var("APPDATA").unwrap_or_default();
    let temp = env::var("TEMP").unwrap_or_default();
    let userprofile = env::var("USERPROFILE").unwrap_or_default();

    push_dir(out, DiskCategory::System, "User Temp", &temp, DiskSafety::Safe, "Temporary files recreated by apps as needed.");
    push_dir(out, DiskCategory::System, "Local Temp", &format!(r"{local}\Temp"), DiskSafety::Safe, "Local temporary files safe to remove while apps are closed.");
    push_dir(out, DiskCategory::System, "Windows Temp", r"C:\Windows\Temp", DiskSafety::Caution, "System temp folder. Close apps before cleaning.");

    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Cache"), DiskSafety::Safe, "Browser cache only. Pages may reload slower after cleanup.");
    push_dir(out, DiskCategory::Browsers, "Chrome Code Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Code Cache"), DiskSafety::Safe, "Cached scripts and assets for Chrome.");
    push_dir(out, DiskCategory::Browsers, "Edge Cache", &format!(r"{local}\Microsoft\Edge\User Data\Default\Cache"), DiskSafety::Safe, "Edge browser cache files.");
    push_dir(out, DiskCategory::Browsers, "Firefox Cache", &format!(r"{local}\Mozilla\Firefox\Profiles"), DiskSafety::Caution, "Firefox profile caches. Browsing data may reset partially.");

    push_dir(out, DiskCategory::Development, "npm Cache", &format!(r"{appdata}\npm-cache"), DiskSafety::Safe, "npm download cache. Packages will re-download when needed.");
    push_dir(out, DiskCategory::Development, "npm Cache (alt)", &format!(r"{userprofile}\.npm\_cacache"), DiskSafety::Safe, "Legacy npm cache location.");
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!(r"{local}\Yarn\Cache"), DiskSafety::Safe, "Yarn package cache.");
    push_dir(out, DiskCategory::Development, "pip Cache", &format!(r"{local}\pip\Cache"), DiskSafety::Safe, "Python pip wheel cache.");
    push_dir(out, DiskCategory::Development, "Cargo Registry Cache", &format!(r"{userprofile}\.cargo\registry\cache"), DiskSafety::Safe, "Rust crate download cache.");
    push_dir(out, DiskCategory::Development, "Cargo Target Cache", &format!(r"{userprofile}\.cargo\registry\src"), DiskSafety::Caution, "Extracted crate sources. May slow next builds slightly.");

    push_dir(out, DiskCategory::Applications, "Discord Cache", &format!(r"{appdata}\discord\Cache"), DiskSafety::Safe, "Discord media and UI cache.");
    push_dir(out, DiskCategory::Applications, "Spotify Cache", &format!(r"{local}\Spotify\Data"), DiskSafety::Caution, "Spotify offline cache and data.");
    push_dir(out, DiskCategory::Applications, "Teams Cache", &format!(r"{appdata}\Microsoft\Teams\Cache"), DiskSafety::Safe, "Microsoft Teams cache files.");
    push_dir(out, DiskCategory::Applications, "VS Code Cache", &format!(r"{appdata}\Code\Cache"), DiskSafety::Safe, "VS Code UI and extension cache.");
    push_dir(out, DiskCategory::Applications, "VS Code CachedData", &format!(r"{appdata}\Code\CachedData"), DiskSafety::Safe, "VS Code cached editor data.");

    push_dir(out, DiskCategory::Storage, "Thumbnail Cache", &format!(r"{local}\Microsoft\Windows\Explorer"), DiskSafety::Caution, "Windows thumbnail previews. They rebuild over time.");
    push_dir(out, DiskCategory::Storage, "Delivery Optimization", &format!(r"{local}\Microsoft\Windows\DeliveryOptimization\Cache"), DiskSafety::Safe, "Windows update delivery cache.");

    push_dir(out, DiskCategory::Other, "DirectX Shader Cache", &format!(r"{local}\D3DSCache"), DiskSafety::Safe, "Regenerated GPU shader cache.");
    push_dir(out, DiskCategory::Other, "Crash Dumps", &format!(r"{local}\CrashDumps"), DiskSafety::Safe, "Application crash dump files.");
    push_dir(out, DiskCategory::Other, "Internet Cache", &format!(r"{local}\Microsoft\Windows\INetCache"), DiskSafety::Safe, "Legacy Windows internet cache.");
}

#[cfg(target_os = "macos")]
fn macos_targets(out: &mut Vec<ScanTarget>) {
    let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

    push_dir(out, DiskCategory::System, "User Caches", &format!("{home}/Library/Caches"), DiskSafety::Caution, "App cache folder. Apps rebuild caches as needed.");
    push_dir(out, DiskCategory::System, "User Logs", &format!("{home}/Library/Logs"), DiskSafety::Safe, "Application log files.");

    push_dir(out, DiskCategory::Browsers, "Safari Cache", &format!("{home}/Library/Caches/com.apple.Safari"), DiskSafety::Safe, "Safari web cache.");
    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!("{home}/Library/Caches/Google/Chrome"), DiskSafety::Safe, "Chrome browser cache.");
    push_dir(out, DiskCategory::Browsers, "Firefox Cache", &format!("{home}/Library/Caches/Firefox"), DiskSafety::Safe, "Firefox browser cache.");

    push_dir(out, DiskCategory::Development, "npm Cache", &format!("{home}/.npm/_cacache"), DiskSafety::Safe, "npm download cache.");
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!("{home}/Library/Caches/Yarn"), DiskSafety::Safe, "Yarn package cache.");
    push_dir(out, DiskCategory::Development, "pip Cache", &format!("{home}/Library/Caches/pip"), DiskSafety::Safe, "Python pip cache.");
    push_dir(out, DiskCategory::Development, "Cargo Cache", &format!("{home}/.cargo/registry/cache"), DiskSafety::Safe, "Rust crate download cache.");
    push_dir(out, DiskCategory::Development, "Xcode DerivedData", &format!("{home}/Library/Developer/Xcode/DerivedData"), DiskSafety::Caution, "Xcode build artifacts. Projects rebuild on next compile.");
    push_dir(out, DiskCategory::Development, "Xcode Archives", &format!("{home}/Library/Developer/Xcode/Archives"), DiskSafety::Caution, "Old Xcode archives. Verify before deleting.");
    push_dir(out, DiskCategory::Development, "Homebrew Cache", &format!("{home}/Library/Caches/Homebrew"), DiskSafety::Safe, "Homebrew download cache.");

    push_dir(out, DiskCategory::Applications, "Discord Cache", &format!("{home}/Library/Application Support/discord/Cache"), DiskSafety::Safe, "Discord cache files.");
    push_dir(out, DiskCategory::Applications, "Slack Cache", &format!("{home}/Library/Application Support/Slack/Cache"), DiskSafety::Safe, "Slack cache files.");
    push_dir(out, DiskCategory::Applications, "VS Code Cache", &format!("{home}/Library/Application Support/Code/Cache"), DiskSafety::Safe, "VS Code cache files.");
    push_dir(out, DiskCategory::Applications, "Spotify Cache", &format!("{home}/Library/Caches/com.spotify.client"), DiskSafety::Caution, "Spotify offline cache.");

    push_dir(out, DiskCategory::Storage, "Mail Downloads", &format!("{home}/Library/Containers/com.apple.mail/Data/Library/Mail Downloads"), DiskSafety::Caution, "Downloaded mail attachments.");
    push_dir(out, DiskCategory::Storage, "iOS Backups", &format!("{home}/Library/Application Support/MobileSync/Backup"), DiskSafety::Unsafe, "Device backups. Only delete if you have copies elsewhere.");

    push_dir(out, DiskCategory::Other, "QuickLook Cache", &format!("{home}/Library/Caches/com.apple.QuickLook"), DiskSafety::Safe, "Preview thumbnail cache.");
    push_dir(out, DiskCategory::Other, "Font Cache", &format!("{home}/Library/Caches/com.apple.ATS"), DiskSafety::Safe, "Font rendering cache.");
}

fn push_dir(
    out: &mut Vec<ScanTarget>,
    category: DiskCategory,
    name: &str,
    path: &str,
    safety: DiskSafety,
    reason: &str,
) {
    if path.is_empty() {
        return;
    }

    out.push(ScanTarget {
        category,
        name: name.to_string(),
        path: path.to_string(),
        safety,
        safety_reason: reason.to_string(),
    });
}

pub fn primary_mount_path() -> String {
    #[cfg(target_os = "windows")]
    {
        return crate::metrics::disk::windows_system_drive();
    }

    #[cfg(not(target_os = "windows"))]
    {
        "/".to_string()
    }
}
