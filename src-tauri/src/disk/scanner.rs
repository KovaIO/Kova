use std::path::{Path, PathBuf};

use crate::disk::models::{DiskCategory, DiskSafety, ScanTarget};

/// Cache subdirectory names commonly used by Windows applications.
const WIN_CACHE_DIRS: &[&str] = &[
    "Cache",
    "GPUCache",
    "CachedData",
    "Code Cache",
    "Service Worker",
    "logs",
    "Code",
    "CachedExtensions",
    "CachedExtensionVSIXs",
];

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

// ═══════════════════════════════════════════════════════════════
//  Windows
// ═══════════════════════════════════════════════════════════════

#[cfg(target_os = "windows")]
fn windows_targets(out: &mut Vec<ScanTarget>) {
    use std::env;

    let local = env::var("LOCALAPPDATA").unwrap_or_default();
    let appdata = env::var("APPDATA").unwrap_or_default();
    let temp = env::var("TEMP").unwrap_or_default();
    let userprofile = env::var("USERPROFILE").unwrap_or_default();

    // ── System (hardcoded) ──────────────────────────────────
    push_dir(out, DiskCategory::System, "User Temp", &temp, DiskSafety::Safe, "Temporary files recreated by apps as needed.");
    push_dir(out, DiskCategory::System, "Local Temp", &format!(r"{local}\Temp"), DiskSafety::Safe, "Local temporary files safe to remove while apps are closed.");
    push_dir(out, DiskCategory::System, "Windows Temp", r"C:\Windows\Temp", DiskSafety::Caution, "System temp folder. Close apps before cleaning.");
    push_dir(out, DiskCategory::System, "Windows Prefetch", r"C:\Windows\Prefetch", DiskSafety::Caution, "Prefetch cache. Windows rebuilds it as you launch apps.");
    push_dir(out, DiskCategory::System, "Windows SoftwareDistribution", r"C:\Windows\SoftwareDistribution\Download", DiskSafety::Caution, "Downloaded Windows Update files. Safe after updates are installed.");

    // ── Browsers (hardcoded — browser paths are well-known) ──
    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Cache"), DiskSafety::Safe, "Browser cache only. Pages may reload slower after cleanup.");
    push_dir(out, DiskCategory::Browsers, "Chrome Code Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Code Cache"), DiskSafety::Safe, "Cached scripts and assets for Chrome.");
    push_dir(out, DiskCategory::Browsers, "Chrome Service Worker", &format!(r"{local}\Google\Chrome\User Data\Default\Service Worker\CacheStorage"), DiskSafety::Safe, "Chrome service worker cache.");
    push_dir(out, DiskCategory::Browsers, "Edge Cache", &format!(r"{local}\Microsoft\Edge\User Data\Default\Cache"), DiskSafety::Safe, "Edge browser cache files.");
    push_dir(out, DiskCategory::Browsers, "Edge Code Cache", &format!(r"{local}\Microsoft\Edge\User Data\Default\Code Cache"), DiskSafety::Safe, "Cached scripts and assets for Edge.");
    push_dir(out, DiskCategory::Browsers, "Firefox Cache", &format!(r"{userprofile}\.cache\mozilla\firefox"), DiskSafety::Safe, "Firefox disk cache.");
    push_dir(out, DiskCategory::Browsers, "Firefox Profiles Cache", &format!(r"{appdata}\Mozilla\Firefox\Profiles"), DiskSafety::Caution, "Firefox profile caches. Browsing data may reset partially.");
    push_dir(out, DiskCategory::Browsers, "Brave Cache", &format!(r"{local}\BraveSoftware\Brave-Browser\User Data\Default\Cache"), DiskSafety::Safe, "Brave browser cache.");
    push_dir(out, DiskCategory::Browsers, "Brave Code Cache", &format!(r"{local}\BraveSoftware\Brave-Browser\User Data\Default\Code Cache"), DiskSafety::Safe, "Brave cached scripts and assets.");
    push_dir(out, DiskCategory::Browsers, "Opera Cache", &format!(r"{appdata}\Opera Software\Opera Stable\Cache"), DiskSafety::Safe, "Opera browser cache.");
    push_dir(out, DiskCategory::Browsers, "Vivaldi Cache", &format!(r"{local}\Vivaldi\User Data\Default\Cache"), DiskSafety::Safe, "Vivaldi browser cache.");

    // ── Development (hardcoded — dev tool paths are specific) ──
    push_dir(out, DiskCategory::Development, "npm Cache", &format!(r"{appdata}\npm-cache"), DiskSafety::Safe, "npm download cache. Packages will re-download when needed.");
    push_dir(out, DiskCategory::Development, "npm Cache (alt)", &format!(r"{userprofile}\.npm\_cacache"), DiskSafety::Safe, "Legacy npm cache location.");
    push_dir(out, DiskCategory::Development, "pnpm Cache", &format!(r"{local}\pnpm\cache"), DiskSafety::Safe, "pnpm download cache.");
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!(r"{local}\Yarn\Cache"), DiskSafety::Safe, "Yarn package cache.");
    push_dir(out, DiskCategory::Development, "pip Cache", &format!(r"{local}\pip\Cache"), DiskSafety::Safe, "Python pip wheel cache.");
    push_dir(out, DiskCategory::Development, "pip HTTP Cache", &format!(r"{local}\pip\http"), DiskSafety::Safe, "Python pip HTTP cache.");
    push_dir(out, DiskCategory::Development, "Cargo Registry Cache", &format!(r"{userprofile}\.cargo\registry\cache"), DiskSafety::Safe, "Rust crate download cache.");
    push_dir(out, DiskCategory::Development, "Cargo Target Cache", &format!(r"{userprofile}\.cargo\registry\src"), DiskSafety::Caution, "Extracted crate sources. May slow next builds slightly.");
    push_dir(out, DiskCategory::Development, "Gradle Cache", &format!(r"{userprofile}\.gradle\caches"), DiskSafety::Safe, "Gradle build cache. Downloads re-fetched on next build.");
    push_dir(out, DiskCategory::Development, "NuGet Cache", &format!(r"{local}\NuGet\v3-cache"), DiskSafety::Safe, "NuGet package cache.");
    push_dir(out, DiskCategory::Development, "Maven Cache", &format!(r"{userprofile}\.m2\repository"), DiskSafety::Caution, "Maven local repository. Re-downloaded as needed.");
    push_dir(out, DiskCategory::Development, "Go Build Cache", &format!(r"{userprofile}\.cache\go-build"), DiskSafety::Safe, "Go build cache.");

    // ── Applications (dynamic) ──────────────────────────────
    let mut seen = std::collections::HashSet::new();
    discover_windows_app_caches(out, &local, &mut seen);
    discover_windows_app_caches(out, &appdata, &mut seen);

    // ── Storage (hardcoded) ─────────────────────────────────
    push_dir(out, DiskCategory::Storage, "Thumbnail Cache", &format!(r"{local}\Microsoft\Windows\Explorer"), DiskSafety::Caution, "Windows thumbnail previews. They rebuild over time.");
    push_dir(out, DiskCategory::Storage, "Delivery Optimization", &format!(r"{local}\Microsoft\Windows\DeliveryOptimization\Cache"), DiskSafety::Safe, "Windows update delivery cache.");
    push_dir(out, DiskCategory::Storage, "Windows.old", r"C:\Windows.old", DiskSafety::Caution, "Previous Windows installation. Keep if you might roll back.");
    push_dir(out, DiskCategory::Storage, "Downloads Cleanup", &format!(r"{userprofile}\Downloads"), DiskSafety::Caution, "User Downloads folder. Review before deleting.");

    // ── Other (hardcoded) ───────────────────────────────────
    push_dir(out, DiskCategory::Other, "DirectX Shader Cache", &format!(r"{local}\D3DSCache"), DiskSafety::Safe, "Regenerated GPU shader cache.");
    push_dir(out, DiskCategory::Other, "Crash Dumps", &format!(r"{local}\CrashDumps"), DiskSafety::Safe, "Application crash dump files.");
    push_dir(out, DiskCategory::Other, "Internet Cache", &format!(r"{local}\Microsoft\Windows\INetCache"), DiskSafety::Safe, "Legacy Windows internet cache.");
    push_dir(out, DiskCategory::Other, "DNS Cache", &format!(r"{local}\DnsClientCache"), DiskSafety::Safe, "DNS resolver cache. Cleared on restart anyway.");
    push_dir(out, DiskCategory::Other, "Icon Cache", &format!(r"{local}\IconCache.db"), DiskSafety::Safe, "Windows icon cache. Rebuilt automatically.");
    push_dir(out, DiskCategory::Other, "Font Cache", r"C:\Windows\ServiceProfiles\LocalService\AppData\Local\FontCache", DiskSafety::Safe, "Windows font cache files.");
}

/// Scans a parent directory (e.g. LOCALAPPDATA or APPDATA) and discovers
/// application cache subdirectories automatically.
#[cfg(target_os = "windows")]
fn discover_windows_app_caches(out: &mut Vec<ScanTarget>, parent: &str, seen: &mut std::collections::HashSet<String>) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };

    for entry in entries.flatten() {
        let app_path = entry.path();
        if !app_path.is_dir() {
            continue;
        }
        let app_name = entry.file_name().to_string_lossy().to_string();

        // Skip well-known system directories that are already handled
        if matches!(
            app_name.as_str(),
            "Microsoft" | "Google" | "Mozilla" | "BraveSoftware"
                | "Opera Software" | "Vivaldi" | "Temp" | "CrashDumps"
                | "D3DSCache" | "pip" | "pnpm" | "Yarn" | "NuGet"
        ) {
            continue;
        }

        for cache_name in WIN_CACHE_DIRS {
            let cache_path = app_path.join(cache_name);
            if !cache_path.exists() || !cache_path.is_dir() {
                continue;
            }

            let path_str = cache_path.to_string_lossy().to_string();
            let key = path_to_key(Path::new(&path_str));
            if !seen.insert(key) {
                continue;
            }

            let label = if *cache_name == "Cache" || *cache_name == "cache" {
                format!("{app_name} Cache")
            } else {
                format!("{app_name} {cache_name}")
            };

            push_dir(
                out,
                DiskCategory::Applications,
                &label,
                &path_str,
                DiskSafety::Safe,
                "Discovered application cache. Safe to clean while the app is closed.",
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  macOS
// ═══════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
fn macos_targets(out: &mut Vec<ScanTarget>) {
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // ── System (hardcoded) ──────────────────────────────────
    push_dir(out, DiskCategory::System, "User Caches", &format!("{home}/Library/Caches"), DiskSafety::Caution, "App cache folder. Apps rebuild caches as needed.");
    push_dir(out, DiskCategory::System, "User Logs", &format!("{home}/Library/Logs"), DiskSafety::Safe, "Application log files.");
    push_dir(out, DiskCategory::System, "Cocoa Cache", &format!("{home}/Library/Caches/com.apple.Cocoa"), DiskSafety::Safe, "Cocoa framework cache.");
    push_dir(out, DiskCategory::System, "CoreServices Cache", &format!("{home}/Library/Caches/com.apple.CoreServices"), DiskSafety::Safe, "macOS CoreServices cache.");
    push_dir(out, DiskCategory::System, "System Diagnostics", &format!("{home}/Library/Logs/DiagnosticReports"), DiskSafety::Safe, "System diagnostic and crash reports.");

    // ── Browsers (hardcoded — browser paths are well-known) ──
    push_dir(out, DiskCategory::Browsers, "Safari Cache", &format!("{home}/Library/Caches/com.apple.Safari"), DiskSafety::Safe, "Safari web cache.");
    push_dir(out, DiskCategory::Browsers, "Safari SafeBrowsing", &format!("{home}/Library/Caches/com.apple.Safari/SafeBrowsing"), DiskSafety::Safe, "Safari safe browsing data cache.");
    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!("{home}/Library/Caches/Google/Chrome"), DiskSafety::Safe, "Chrome browser cache.");
    push_dir(out, DiskCategory::Browsers, "Chrome Code Cache", &format!("{home}/Library/Caches/Google/Chrome/Code Cache"), DiskSafety::Safe, "Chrome code cache.");
    push_dir(out, DiskCategory::Browsers, "Edge Cache", &format!("{home}/Library/Caches/Microsoft Edge"), DiskSafety::Safe, "Edge browser cache.");
    push_dir(out, DiskCategory::Browsers, "Firefox Cache", &format!("{home}/Library/Caches/Firefox"), DiskSafety::Safe, "Firefox browser cache.");
    push_dir(out, DiskCategory::Browsers, "Brave Cache", &format!("{home}/Library/Caches/BraveSoftware"), DiskSafety::Safe, "Brave browser cache.");
    push_dir(out, DiskCategory::Browsers, "Opera Cache", &format!("{home}/Library/Caches/com.operasoftware.Opera"), DiskSafety::Safe, "Opera browser cache.");
    push_dir(out, DiskCategory::Browsers, "Vivaldi Cache", &format!("{home}/Library/Caches/Vivaldi"), DiskSafety::Safe, "Vivaldi browser cache.");

    // ── Development (hardcoded — dev tool paths are specific) ──
    push_dir(out, DiskCategory::Development, "npm Cache", &format!("{home}/.npm/_cacache"), DiskSafety::Safe, "npm download cache.");
    push_dir(out, DiskCategory::Development, "pnpm Cache", &format!("{home}/Library/pnpm/store"), DiskSafety::Safe, "pnpm content-addressable store.");
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!("{home}/Library/Caches/Yarn"), DiskSafety::Safe, "Yarn package cache.");
    push_dir(out, DiskCategory::Development, "pip Cache", &format!("{home}/Library/Caches/pip"), DiskSafety::Safe, "Python pip cache.");
    push_dir(out, DiskCategory::Development, "Cargo Cache", &format!("{home}/.cargo/registry/cache"), DiskSafety::Safe, "Rust crate download cache.");
    push_dir(out, DiskCategory::Development, "Cargo Git Cache", &format!("{home}/.cargo/git/db"), DiskSafety::Safe, "Rust cargo git checkout cache.");
    push_dir(out, DiskCategory::Development, "Gradle Cache", &format!("{home}/.gradle/caches"), DiskSafety::Safe, "Gradle build cache.");
    push_dir(out, DiskCategory::Development, "Maven Cache", &format!("{home}/.m2/repository"), DiskSafety::Caution, "Maven local repository. Re-downloaded as needed.");
    push_dir(out, DiskCategory::Development, "Go Build Cache", &format!("{home}/Library/Caches/go-build"), DiskSafety::Safe, "Go build cache.");
    push_dir(out, DiskCategory::Development, "Homebrew Cache", &format!("{home}/Library/Caches/Homebrew"), DiskSafety::Safe, "Homebrew download cache.");
    push_dir(out, DiskCategory::Development, "Xcode DerivedData", &format!("{home}/Library/Developer/Xcode/DerivedData"), DiskSafety::Caution, "Xcode build artifacts. Projects rebuild on next compile.");
    push_dir(out, DiskCategory::Development, "Xcode Archives", &format!("{home}/Library/Developer/Xcode/Archives"), DiskSafety::Caution, "Old Xcode archives. Verify before deleting.");
    push_dir(out, DiskCategory::Development, "Xcode iOS DeviceSupport", &format!("{home}/Library/Developer/Xcode/iOS DeviceSupport"), DiskSafety::Caution, "iOS device support files. Needed for on-device debugging.");
    push_dir(out, DiskCategory::Development, "Android SDK Cache", &format!("{home}/Library/Android/sdk/.temp"), DiskSafety::Safe, "Android SDK temporary build files.");
    push_dir(out, DiskCategory::Development, "Docker Cache", &format!("{home}/Library/Containers/com.docker.docker/Data/vms"), DiskSafety::Safe, "Docker VM disk images cache.");
    push_dir(out, DiskCategory::Development, "Bun Cache", &format!("{home}/.bun/install/cache"), DiskSafety::Safe, "Bun package manager cache.");

    // ── Applications (dynamic) ──────────────────────────────
    discover_macos_app_caches(out, &home);

    // ── Storage (hardcoded) ─────────────────────────────────
    push_dir(out, DiskCategory::Storage, "Mail Downloads", &format!("{home}/Library/Containers/com.apple.mail/Data/Library/Mail Downloads"), DiskSafety::Caution, "Downloaded mail attachments.");
    push_dir(out, DiskCategory::Storage, "iOS Backups", &format!("{home}/Library/Application Support/MobileSync/Backup"), DiskSafety::Unsafe, "Device backups. Only delete if you have copies elsewhere.");
    push_dir(out, DiskCategory::Storage, "Messages Attachments", &format!("{home}/Library/Messages/Attachments"), DiskSafety::Caution, "iMessage and SMS attachments.");
    push_dir(out, DiskCategory::Storage, "Downloads Cleanup", &format!("{home}/Downloads"), DiskSafety::Caution, "User Downloads folder. Review before deleting.");

    // ── Other (hardcoded) ───────────────────────────────────
    push_dir(out, DiskCategory::Other, "QuickLook Cache", &format!("{home}/Library/Caches/com.apple.QuickLook"), DiskSafety::Safe, "Preview thumbnail cache.");
    push_dir(out, DiskCategory::Other, "Font Cache", &format!("{home}/Library/Caches/com.apple.ATS"), DiskSafety::Safe, "Font rendering cache.");
    push_dir(out, DiskCategory::Other, "Spotlight Cache", &format!("{home}/Library/Caches/com.apple.Spotlight"), DiskSafety::Safe, "Spotlight search index cache.");
    push_dir(out, DiskCategory::Other, "UIServer Cache", &format!("{home}/Library/Caches/com.apple.WindowManager"), DiskSafety::Safe, "Window Manager cache.");
}

/// Scans ~/Library/Caches and ~/Library/Application Support/*/Cache to
/// discover application caches dynamically.
#[cfg(target_os = "macos")]
fn discover_macos_app_caches(out: &mut Vec<ScanTarget>, home: &str) {
    let mut seen = std::collections::HashSet::new();

    // ── ~/Library/Caches/* ──────────────────────────────────
    let caches_dir = format!("{home}/Library/Caches");
    if let Ok(entries) = std::fs::read_dir(&caches_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip Apple system caches and well-known dev/browser dirs
            if name.starts_with("com.apple.")
                || name.starts_with("com.microsoft.")
                || matches!(
                    name.as_str(),
                    "Google" | "Firefox" | "BraveSoftware" | "Vivaldi"
                        | "com.operasoftware.Opera" | "Homebrew" | "Yarn"
                        | "pip" | "go-build"
                )
            {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let key = path_to_key(Path::new(&path_str));
            if seen.insert(key) {
                push_dir(
                    out,
                    DiskCategory::Applications,
                    &format!("{name} Cache"),
                    &path_str,
                    DiskSafety::Safe,
                    "Discovered application cache. Safe to clean.",
                );
            }
        }
    }

    // ── ~/Library/Application Support/*/Cache ────────────────
    let support_dir = format!("{home}/Library/Application Support");
    if let Ok(entries) = std::fs::read_dir(&support_dir) {
        for entry in entries.flatten() {
            let app_path = entry.path();
            if !app_path.is_dir() {
                continue;
            }
            let app_name = entry.file_name().to_string_lossy().to_string();

            // Skip Apple system and already-handled dirs
            if app_name.starts_with("com.apple.")
                || matches!(
                    app_name.as_str(),
                    "Google" | "Firefox" | "BraveSoftware" | "Slack"
                        | "Code" | "discord" | "Spotify" | "Notion"
                        | "Figma" | "1Password" | "Microsoft"
                )
            {
                continue;
            }

            let cache_path = app_path.join("Cache");
            if cache_path.exists() && cache_path.is_dir() {
                let path_str = cache_path.to_string_lossy().to_string();
                let key = path_to_key(Path::new(&path_str));
                if seen.insert(key) {
                    push_dir(
                        out,
                        DiskCategory::Applications,
                        &format!("{app_name} Cache"),
                        &path_str,
                        DiskSafety::Safe,
                        "Discovered application cache. Safe to clean.",
                    );
                }
            }
        }
    }
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
