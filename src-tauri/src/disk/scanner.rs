use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::disk::models::{
    CleanupAction, DiskCategory, DiskSafety, ScanTarget, ScanTargetKind, VirtualTarget,
};

/// Cache subdirectory names commonly used by Electron-based apps.
/// NOTE: "Code" is intentionally excluded — many apps use it for real data,
/// not cache. Only "Code Cache" is safe.
/// NOTE: "Service Worker" and "CacheStorage" are excluded — they contain
/// offline state, auth tokens, and app data that shouldn't be auto-deleted.
const WIN_CACHE_DIRS: &[&str] = &[
    "Cache",
    "GPUCache",
    "CachedData",
    "Code Cache",
    "Crashpad",
    "CrashReports",
    "logs",
    "CachedExtensions",
    "CachedExtensionVSIXs",
    "DerivedDataCache",
    "ShaderCache",
    "WebCache",
];

/// Electron-based IDEs — we enumerate their cache subdirectories under APPDATA.
const ELECTRON_IDES: &[&str] = &[
    "Code",
    "Code - Insiders",
    "VSCodium",
    "Cursor",
    "Windsurf",
    "Arc",
    "Zed",
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
    let mut total = 0u64;
    let mut count = 0u32;

    let walker = WalkDir::new(path)
        .max_depth(max_depth as usize)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.file_name()));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
                count += 1;
            }
        }
    }

    (total, count)
}

/// Faster dir_size with no depth limit — for Recycle Bin, etc.
pub fn dir_size_unlimited(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

pub fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Compute size of files matching a glob pattern inside a parent directory.
pub fn pattern_size(parent: &str, pattern: &str) -> (u64, u32) {
    let parent_path = Path::new(parent);
    if !parent_path.is_dir() {
        return (0, 0);
    }

    let glob_pattern = format!(
        "{}/{}",
        parent_path.to_string_lossy().replace('\\', "/"),
        pattern
    );

    let mut total = 0u64;
    let mut count = 0u32;

    if let Ok(entries) = glob::glob(&glob_pattern) {
        for entry in entries.flatten() {
            if let Ok(meta) = std::fs::metadata(&entry) {
                total += meta.len();
                count += 1;
            }
        }
    }

    (total, count)
}

/// Get individual file paths matching a pattern (for deletion).
pub fn pattern_files(parent: &str, pattern: &str) -> Vec<PathBuf> {
    let parent_path = Path::new(parent);
    if !parent_path.is_dir() {
        return vec![];
    }

    let glob_pattern = format!(
        "{}/{}",
        parent_path.to_string_lossy().replace('\\', "/"),
        pattern
    );

    glob::glob(&glob_pattern)
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .collect()
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
    let Ok(canonical) = path.canonicalize() else {
        return false;
    };

    allowed_roots.iter().any(|root| {
        let root_path = Path::new(root);
        let Ok(canonical_root) = root_path.canonicalize() else {
            return false;
        };
        canonical.starts_with(&canonical_root)
    })
}

fn path_to_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

// ═══════════════════════════════════════════════════════════════
//  Process detection
// ═══════════════════════════════════════════════════════════════

/// Returns true if any of the given process names are currently running.
pub fn is_any_process_running(processes: &[String]) -> bool {
    if processes.is_empty() {
        return false;
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("tasklist")
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // CSV format: "Image Name","PID","Session Name","Session#","Mem Usage"
            // /NH omits the header row
            let wanted: Vec<&str> = processes.iter().map(|s| s.as_str()).collect();
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                // Parse first CSV field: the image name (quoted)
                let image_name = parse_first_csv_field(line);
                if !image_name.is_empty()
                    && wanted.iter().any(|w| image_name.eq_ignore_ascii_case(w))
                {
                    return true;
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("ps").arg("axo").arg("comm").output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                // comm is the bare executable name, e.g. "Google Chrome"
                if processes
                    .iter()
                    .any(|p| line.eq_ignore_ascii_case(p))
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Parse the first field of a CSV line, handling quoted fields.
/// Input: `"chrome.exe","12345","Console","1","123,456 K"`
/// Output: `chrome.exe`
#[cfg(target_os = "windows")]
fn parse_first_csv_field(line: &str) -> &str {
    let line = line.trim_start();
    if let Some(rest) = line.strip_prefix('"') {
        // Quoted field — find the closing quote
        if let Some(end) = rest.find('"') {
            &rest[..end]
        } else {
            // Malformed — take the rest
            rest
        }
    } else {
        // Unquoted field — take up to the next comma
        line.split(',').next().unwrap_or("")
    }
}

// ═══════════════════════════════════════════════════════════════
//  Virtual target implementations
// ═══════════════════════════════════════════════════════════════

/// Estimate Recycle Bin size by scanning all recycle bin directories.
#[cfg(target_os = "windows")]
pub fn recycle_bin_size() -> u64 {
    let mut total = 0u64;

    // Scan all mounted drives — each has its own $Recycle.Bin
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:", letter as char);
        let root = format!("{}\\", drive);
        // Check if drive exists and is ready
        if std::fs::metadata(&root).is_ok() {
            drives.push(drive);
        }
    }

    for drive in &drives {
        let recycle_root = format!("{}\\$Recycle.Bin", drive);
        if let Ok(entries) = std::fs::read_dir(&recycle_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let size = dir_size_unlimited(&path);
                    total += size;
                }
            }
        }
    }

    total
}

#[cfg(not(target_os = "windows"))]
pub fn recycle_bin_size() -> u64 {
    0
}

/// Empty the Recycle Bin on Windows.
#[cfg(target_os = "windows")]
pub fn empty_recycle_bin() -> Result<u64, String> {
    use windows::Win32::UI::Shell::SHEmptyRecycleBinW;
    use windows::core::PCWSTR;

    let before = recycle_bin_size();

    // SHERB_NOCONFIRMATION (0x1) | SHERB_NOPROGRESSUI (0x2) | SHERB_NOSOUND (0x4)
    let flags = 0x7u32;

    let result = unsafe {
        SHEmptyRecycleBinW(
            None,
            PCWSTR::null(),
            flags,
        )
    };

    match result {
        Ok(()) => Ok(before),
        Err(e) => {
            // S_FALSE means the bin was already empty — still a success
            if e.code().0 as u32 == 0x40010002 {
                Ok(before)
            } else {
                Err(format!("SHEmptyRecycleBinW failed: {}", e))
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn empty_recycle_bin() -> Result<u64, String> {
    Err("Recycle bin not supported on this OS".into())
}

/// Find all WSL VHDX file paths.
#[cfg(target_os = "windows")]
fn find_wsl_vhdx_paths() -> Vec<String> {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let packages = format!("{local}\\Packages");
    let mut paths = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&packages) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let local_state = path.join("LocalState");
            if local_state.is_dir() {
                if let Ok(ls_entries) = std::fs::read_dir(&local_state) {
                    for ls_entry in ls_entries.flatten() {
                        let ls_name = ls_entry.file_name().to_string_lossy().to_string();
                        if ls_name.to_lowercase() == "ext4.vhdx" {
                            paths.push(ls_entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    paths
}

#[cfg(not(target_os = "windows"))]
fn find_wsl_vhdx_paths() -> Vec<String> {
    vec![]
}

/// Find Docker VHDX file paths.
#[cfg(target_os = "windows")]
fn find_docker_vhdx_paths() -> Vec<String> {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let docker_path = format!("{local}\\Docker");
    let mut paths = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&docker_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.to_lowercase().ends_with(".vhdx") {
                paths.push(path.to_string_lossy().to_string());
            }
        }
    }

    paths
}

#[cfg(not(target_os = "windows"))]
fn find_docker_vhdx_paths() -> Vec<String> {
    vec![]
}

// ═══════════════════════════════════════════════════════════════
//  Windows — Tier 1 (Safe) targets
// ═══════════════════════════════════════════════════════════════

#[cfg(target_os = "windows")]
fn windows_targets(out: &mut Vec<ScanTarget>) {
    use std::env;

    let local = env::var("LOCALAPPDATA").unwrap_or_default();
    let appdata = env::var("APPDATA").unwrap_or_default();
    let temp = env::var("TEMP").unwrap_or_default();
    let userprofile = env::var("USERPROFILE").unwrap_or_default();

    // ── System ──────────────────────────────────────────────
    push_dir(out, DiskCategory::System, "User Temp", &temp, DiskSafety::Safe, "Temporary files recreated by apps as needed.", &[]);
    push_dir(out, DiskCategory::System, "Windows Temp", r"C:\Windows\Temp", DiskSafety::Safe, "System temp folder. Files in use will fail to delete automatically.", &[]);

    // ── Browsers ────────────────────────────────────────────
    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Cache"), DiskSafety::Safe, "Browser cache only. Pages may reload slower after cleanup.", &["chrome.exe"]);
    push_dir(out, DiskCategory::Browsers, "Chrome Code Cache", &format!(r"{local}\Google\Chrome\User Data\Default\Code Cache"), DiskSafety::Safe, "Cached scripts and assets for Chrome.", &["chrome.exe"]);
    push_dir(out, DiskCategory::Browsers, "Chrome Service Worker", &format!(r"{local}\Google\Chrome\User Data\Default\Service Worker\CacheStorage"), DiskSafety::Safe, "Chrome service worker cache.", &["chrome.exe"]);
    push_dir(out, DiskCategory::Browsers, "Edge Cache", &format!(r"{local}\Microsoft\Edge\User Data\Default\Cache"), DiskSafety::Safe, "Edge browser cache files.", &["msedge.exe"]);
    push_dir(out, DiskCategory::Browsers, "Edge Code Cache", &format!(r"{local}\Microsoft\Edge\User Data\Default\Code Cache"), DiskSafety::Safe, "Cached scripts and assets for Edge.", &["msedge.exe"]);

    // Firefox — enumerate actual profile directories
    discover_firefox_caches(out, &local);

    push_dir(out, DiskCategory::Browsers, "Brave Cache", &format!(r"{local}\BraveSoftware\Brave-Browser\User Data\Default\Cache"), DiskSafety::Safe, "Brave browser cache.", &["brave.exe"]);
    push_dir(out, DiskCategory::Browsers, "Brave Code Cache", &format!(r"{local}\BraveSoftware\Brave-Browser\User Data\Default\Code Cache"), DiskSafety::Safe, "Brave cached scripts and assets.", &["brave.exe"]);
    push_dir(out, DiskCategory::Browsers, "Opera Cache", &format!(r"{appdata}\Opera Software\Opera Stable\Cache"), DiskSafety::Safe, "Opera browser cache.", &["opera.exe"]);
    push_dir(out, DiskCategory::Browsers, "Vivaldi Cache", &format!(r"{local}\Vivaldi\User Data\Default\Cache"), DiskSafety::Safe, "Vivaldi browser cache.", &["vivaldi.exe"]);

    // ── Development — Package managers ───────────────────────
    push_dir(out, DiskCategory::Development, "npm Cache", &format!(r"{appdata}\npm-cache"), DiskSafety::Safe, "npm download cache. Packages will re-download when needed.", &[]);
    push_dir(out, DiskCategory::Development, "npm Cache (alt)", &format!(r"{userprofile}\.npm\_cacache"), DiskSafety::Safe, "Legacy npm cache location.", &[]);
    push_dir(out, DiskCategory::Development, "pnpm Cache", &format!(r"{local}\pnpm\cache"), DiskSafety::Safe, "pnpm download cache.", &[]);
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!(r"{local}\Yarn\Cache"), DiskSafety::Safe, "Yarn package cache.", &[]);
    push_dir(out, DiskCategory::Development, "pip Cache", &format!(r"{local}\pip\Cache"), DiskSafety::Safe, "Python pip wheel cache.", &[]);
    push_dir(out, DiskCategory::Development, "pip HTTP Cache", &format!(r"{local}\pip\http"), DiskSafety::Safe, "Python pip HTTP cache.", &[]);
    push_dir(out, DiskCategory::Development, "Cargo Registry Cache", &format!(r"{userprofile}\.cargo\registry\cache"), DiskSafety::Safe, "Rust crate download cache.", &[]);
    push_dir(out, DiskCategory::Development, "Gradle Cache", &format!(r"{userprofile}\.gradle\caches"), DiskSafety::Caution, "Gradle build cache. Next build may re-download many GB.", &[]);
    push_dir(out, DiskCategory::Development, "NuGet Cache", &format!(r"{local}\NuGet\v3-cache"), DiskSafety::Safe, "NuGet package cache.", &[]);
    push_dir(out, DiskCategory::Development, "Go Build Cache", &format!(r"{userprofile}\.cache\go-build"), DiskSafety::Safe, "Go build cache.", &[]);
    push_dir(out, DiskCategory::Development, "Bun Cache", &format!(r"{userprofile}\.bun\install\cache"), DiskSafety::Safe, "Bun package manager cache.", &[]);
    push_dir(out, DiskCategory::Development, "Conan Cache", &format!(r"{userprofile}\.conan"), DiskSafety::Safe, "Conan C++ package manager cache.", &[]);
    push_dir(out, DiskCategory::Development, "vcpkg Cache", &format!(r"{local}\vcpkg\archives"), DiskSafety::Safe, "vcpkg package archives.", &[]);

    // ── Development — Electron IDEs (generalized) ────────────
    discover_electron_ide_caches(out, &appdata);

    push_dir(out, DiskCategory::Development, "JetBrains Transient", &format!(r"{local}\JetBrains\Transient"), DiskSafety::Safe, "JetBrains IDE transient cache.", &[]);

    // ── Development — JetBrains & Android Studio ─────────────
    discover_jetbrains_and_android_caches(out, &local);

    // ── Development — Game engines ──────────────────────────
    push_dir(out, DiskCategory::Development, "Unreal DerivedDataCache", &format!(r"{local}\UnrealEngine\Common\DerivedDataCache"), DiskSafety::Safe, "Unreal Engine derived data cache. Regenerated automatically.", &["UnrealEditor.exe", "UE4Editor.exe"]);
    push_dir(out, DiskCategory::Development, "Unity Cache", &format!(r"{local}\Unity\cache"), DiskSafety::Safe, "Unity cache. Regenerated automatically.", &["Unity.exe", "Unity Hub.exe"]);

    // ── Applications (dynamic) ──────────────────────────────
    let mut seen = std::collections::HashSet::new();
    discover_windows_app_caches(out, &local, &mut seen);
    discover_windows_app_caches(out, &appdata, &mut seen);

    // ── Storage ─────────────────────────────────────────────
    // Thumbnail cache — file pattern only, not the whole Explorer folder
    push_pattern(
        out,
        DiskCategory::Storage,
        "Thumbnail Cache",
        &format!(r"{local}\Microsoft\Windows\Explorer"),
        "thumbcache_*.db",
        DiskSafety::Caution,
        "Windows thumbnail previews. They rebuild over time.",
    );
    push_dir(out, DiskCategory::Storage, "Delivery Optimization", &format!(r"{local}\Microsoft\Windows\DeliveryOptimization\Cache"), DiskSafety::Safe, "Windows update delivery cache.", &[]);

    // ── Storage — Recycle Bin ───────────────────────────────
    push_virtual(out, DiskCategory::Storage, "Recycle Bin", DiskSafety::Caution, "Empties the Recycle Bin on all drives. Review contents before cleaning.", VirtualTarget::RecycleBin);

    // ── Storage — Docker ────────────────────────────────────
    discover_docker_storage(out, &local);

    // ── Storage — WSL ───────────────────────────────────────
    discover_wsl_storage(out, &local);

    // ── Other ───────────────────────────────────────────────
    push_dir(out, DiskCategory::Other, "DirectX Shader Cache", &format!(r"{local}\D3DSCache"), DiskSafety::Safe, "Regenerated GPU shader cache.", &[]);
    push_dir(out, DiskCategory::Other, "NVIDIA DXCache", &format!(r"{local}\NVIDIA\DXCache"), DiskSafety::Safe, "NVIDIA DirectX shader cache. Regenerated automatically.", &[]);
    push_dir(out, DiskCategory::Other, "NVIDIA GLCache", &format!(r"{local}\NVIDIA\GLCache"), DiskSafety::Safe, "NVIDIA OpenGL shader cache. Regenerated automatically.", &[]);
    push_dir(out, DiskCategory::Other, "AMD DxCache", &format!(r"{local}\AMD\DxCache"), DiskSafety::Safe, "AMD DirectX shader cache. Regenerated automatically.", &[]);
    push_dir(out, DiskCategory::Other, "Crash Dumps", &format!(r"{local}\CrashDumps"), DiskSafety::Safe, "Application crash dump files.", &[]);
    push_dir(out, DiskCategory::Other, "Windows Error Reporting", &format!(r"{local}\Microsoft\Windows\WER"), DiskSafety::Safe, "Windows crash reports and diagnostics.", &[]);
    push_dir(out, DiskCategory::Other, "Internet Cache", &format!(r"{local}\Microsoft\Windows\INetCache"), DiskSafety::Safe, "Legacy Windows internet cache.", &[]);
    push_file(out, DiskCategory::Other, "Icon Cache", &format!(r"{local}\IconCache.db"), DiskSafety::Safe, "Windows icon cache. Rebuilt automatically.", &[]);

    // ── System — MEMORY.DMP ─────────────────────────────────
    push_file(out, DiskCategory::System, "Crash Dump (MEMORY.DMP)", r"C:\Windows\MEMORY.DMP", DiskSafety::Caution, "Kernel crash dump. Can be 10–50 GB. Only needed for crash analysis.", &[]);

    // ── System — Minidumps ──────────────────────────────────
    push_dir(out, DiskCategory::System, "Minidumps", r"C:\Windows\Minidump", DiskSafety::Safe, "Small crash dump files. Safe to clean.", &[]);

    // ── System — Windows.old ────────────────────────────────
    push_dir(out, DiskCategory::System, "Windows.old", r"C:\Windows.old", DiskSafety::Caution, "Previous Windows installation. After cleanup you cannot roll back.", &[]);

    // ── System — Windows Update Download Cache ──────────────
    push_dir(out, DiskCategory::System, "Windows Update Downloads", r"C:\Windows\SoftwareDistribution\Download", DiskSafety::Caution, "Cached Windows Update installers. Only safe to clean when no updates are pending.", &["wuauserv.exe"]);

    // ── Development — Visual Studio ─────────────────────────
    push_dir(out, DiskCategory::Development, "VS Component Model Cache", &format!(r"{local}\Microsoft\VisualStudio\ComponentModelCache"), DiskSafety::Safe, "Visual Studio component model cache. Regenerated on next launch.", &[]);
    push_dir(out, DiskCategory::Development, "VS Image Library", &format!(r"{local}\Microsoft\VisualStudio\ImageLibrary"), DiskSafety::Safe, "Visual Studio image cache. Regenerated on next launch.", &[]);
    push_dir(out, DiskCategory::Development, "VS MEF Cache", &format!(r"{local}\Microsoft\VisualStudio\MefCache"), DiskSafety::Safe, "Visual Studio MEF extension cache.", &[]);

    // ── Storage — Unreal Engine Vault Cache (AnalyzeOnly) ───
    push_file_analyze_only(out, DiskCategory::Storage, "Unreal Vault Cache", &format!(r"{appdata}\Unreal Engine\UnrealEngineVaultCache"), DiskSafety::Caution, "Downloaded Unreal Engine marketplace assets. Do NOT delete — these are your purchased assets.");

    // ── Other — NVIDIA ProgramData cache ────────────────────
    push_dir(out, DiskCategory::Other, "NVIDIA ProgramData Cache", r"C:\ProgramData\NVIDIA Corporation\NV_Cache", DiskSafety::Safe, "NVIDIA driver shader cache. Regenerated automatically.", &[]);
}

/// Scans LOCALAPPDATA for JetBrains product caches (AndroidStudio, IntelliJ, CLion, etc.)
/// and other Google dev tool caches.
#[cfg(target_os = "windows")]
fn discover_jetbrains_and_android_caches(out: &mut Vec<ScanTarget>, local: &str) {
    // JetBrains products live under LOCALAPPDATA\JetBrains\Transient (already added)
    // and under their own vendor directories.
    // Also scan for versioned JetBrains product directories.
    let jetbrains_products = [
        "JetBrains",
    ];

    for vendor in &jetbrains_products {
        let vendor_dir = format!("{local}\\{vendor}");
        let Ok(entries) = std::fs::read_dir(&vendor_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            // Match product names followed by version: IntelliJIDEA2024.1, CLion2024.2, etc.
            let is_jb_product = name.starts_with("IntelliJIDEA")
                || name.starts_with("CLion")
                || name.starts_with("PyCharm")
                || name.starts_with("WebStorm")
                || name.starts_with("Rider")
                || name.starts_with("GoLand")
                || name.starts_with("DataGrip")
                || name.starts_with("PhpStorm")
                || name.starts_with("RubyMine")
                || name.starts_with("AndroidStudio");

            if !is_jb_product {
                continue;
            }

            let caches = path.join("caches");
            if caches.exists() && caches.is_dir() {
                push_dir(
                    out,
                    DiskCategory::Development,
                    &format!("{name} Caches"),
                    &caches.to_string_lossy(),
                    DiskSafety::Safe,
                    "JetBrains IDE cache. Regenerated on next build.",
                    &[],
                );
            }

            let logs = path.join("log");
            if logs.exists() && logs.is_dir() {
                push_dir(
                    out,
                    DiskCategory::Development,
                    &format!("{name} Logs"),
                    &logs.to_string_lossy(),
                    DiskSafety::Safe,
                    "JetBrains IDE log files.",
                    &[],
                );
            }
        }
    }

    // Android Studio lives under LOCALAPPDATA\Google\AndroidStudio*
    let google_dir = format!("{local}\\Google");
    if let Ok(entries) = std::fs::read_dir(&google_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            if !name.starts_with("AndroidStudio") {
                continue;
            }

            let caches = path.join("caches");
            if caches.exists() && caches.is_dir() {
                push_dir(
                    out,
                    DiskCategory::Development,
                    &format!("{name} Caches"),
                    &caches.to_string_lossy(),
                    DiskSafety::Safe,
                    "Android Studio cache. Regenerated on next build.",
                    &[],
                );
            }

            let logs = path.join("log");
            if logs.exists() && logs.is_dir() {
                push_dir(
                    out,
                    DiskCategory::Development,
                    &format!("{name} Logs"),
                    &logs.to_string_lossy(),
                    DiskSafety::Safe,
                    "Android Studio log files.",
                    &[],
                );
            }
        }
    }
}

/// Enumerate Firefox profile cache directories on Windows.
/// Firefox stores profiles in %LOCALAPPDATA%\Mozilla\Firefox\Profiles\
#[cfg(target_os = "windows")]
fn discover_firefox_caches(out: &mut Vec<ScanTarget>, local: &str) {
    let profiles_root = format!("{local}\\Mozilla\\Firefox\\Profiles");
    let Ok(entries) = std::fs::read_dir(&profiles_root) else {
        return;
    };

    for entry in entries.flatten() {
        let profile_dir = entry.path();
        if !profile_dir.is_dir() {
            continue;
        }

        // Firefox profiles are named like: xxxxxxxx.default-release
        let profile_name = profile_dir.file_name().unwrap_or_default().to_string_lossy();

        // disk cache (main cache)
        let cache2 = profile_dir.join("cache2");
        if cache2.exists() && cache2.is_dir() {
            push_dir(
                out,
                DiskCategory::Browsers,
                &format!("Firefox ({profile_name}) Cache"),
                &cache2.to_string_lossy(),
                DiskSafety::Safe,
                "Firefox disk cache for this profile.",
                &["firefox.exe"],
            );
        }

        // startup cache
        let startup = profile_dir.join("startupCache");
        if startup.exists() && startup.is_dir() {
            push_dir(
                out,
                DiskCategory::Browsers,
                &format!("Firefox ({profile_name}) Startup Cache"),
                &startup.to_string_lossy(),
                DiskSafety::Safe,
                "Firefox startup cache.",
                &["firefox.exe"],
            );
        }
    }
}

/// Discover Electron-based IDE cache directories under APPDATA.
/// Handles VS Code, VS Code Insiders, VSCodium, Cursor, Windsurf, etc.
#[cfg(target_os = "windows")]
fn discover_electron_ide_caches(out: &mut Vec<ScanTarget>, appdata: &str) {
    let mut seen = std::collections::HashSet::new();

    for ide_name in ELECTRON_IDES {
        let ide_dir = format!("{appdata}\\{ide_name}");
        if !Path::new(&ide_dir).is_dir() {
            continue;
        }

        for cache_name in WIN_CACHE_DIRS {
            let cache_path = format!("{ide_dir}\\{cache_name}");
            if !Path::new(&cache_path).is_dir() {
                continue;
            }

            let key = path_to_key(Path::new(&cache_path));
            if !seen.insert(key) {
                continue;
            }

            let label = if *cache_name == "Cache" || *cache_name == "cache" {
                format!("{ide_name} Cache")
            } else {
                format!("{ide_name} {cache_name}")
            };

            push_dir(
                out,
                DiskCategory::Development,
                &label,
                &cache_path,
                DiskSafety::Safe,
                &format!("{ide_name} cache. Safe to clean while the IDE is closed."),
                &[],
            );
        }
    }
}

/// Scans a parent directory (e.g. LOCALAPPDATA or APPDATA) and discovers
/// application cache subdirectories automatically.
#[cfg(target_os = "windows")]
fn discover_windows_app_caches(out: &mut Vec<ScanTarget>, parent: &str, seen: &mut std::collections::HashSet<String>) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };

    // Skip directories already handled by explicit targets above
    let skip_dirs = [
        "Microsoft", "Google", "Mozilla", "BraveSoftware",
        "Opera Software", "Vivaldi", "Temp", "CrashDumps",
        "D3DSCache", "pip", "pnpm", "Yarn", "NuGet", "JetBrains",
        "NVIDIA", "AMD", "UnrealEngine", "Unity",
        "Code", "Code - Insiders", "VSCodium", "Cursor",
        "Windsurf", "Arc", "Zed",
    ];

    for entry in entries.flatten() {
        let app_path = entry.path();
        if !app_path.is_dir() {
            continue;
        }
        let app_name = entry.file_name().to_string_lossy().to_string();

        if skip_dirs.contains(&app_name.as_str()) {
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

            // Classify safety by folder name — some look like cache but contain state
            let (safety, reason) = cache_safety_and_reason(cache_name);

            push_dir(
                out,
                DiskCategory::Applications,
                &label,
                &path_str,
                safety,
                reason,
                &[],
            );
        }
    }
}

/// Discover Docker Desktop storage (VHDX files).
#[cfg(target_os = "windows")]
fn discover_docker_storage(out: &mut Vec<ScanTarget>, local: &str) {
    let docker_path = format!("{local}\\Docker");
    if !Path::new(&docker_path).is_dir() {
        return;
    }

    let vhdx_paths = find_docker_vhdx_paths();
    for (i, path) in vhdx_paths.iter().enumerate() {
        let name = if i == 0 {
            "Docker VM Disk".to_string()
        } else {
            format!("Docker VM Disk {}", i + 1)
        };
        push_file_analyze_only(
            out,
            DiskCategory::Storage,
            &name,
            path,
            DiskSafety::Caution,
            "Docker Desktop virtual disk. Do NOT delete — this destroys all containers and images.",
        );
    }

    // Also scan wsl/ subdirectory for additional VHDX
    let wsl_dir = format!("{docker_path}\\wsl");
    if Path::new(&wsl_dir).is_dir() {
        if let Ok(entries) = std::fs::read_dir(&wsl_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_lowercase().ends_with(".vhdx") {
                    let path_str = entry.path().to_string_lossy().to_string();
                    push_file_analyze_only(
                        out,
                        DiskCategory::Storage,
                        &format!("Docker WSL Disk ({name})"),
                        &path_str,
                        DiskSafety::Caution,
                        "Docker WSL virtual disk. Do NOT delete — this destroys all containers.",
                    );
                }
            }
        }
    }
}

/// Discover WSL distro VHDX files.
#[cfg(target_os = "windows")]
fn discover_wsl_storage(out: &mut Vec<ScanTarget>, _local: &str) {
    let vhdx_paths = find_wsl_vhdx_paths();
    for path in vhdx_paths {
        // Extract distro name from path: ...Packages\CanonicalGroupLimited.Ubuntu*\LocalState\ext4.vhdx
        let distro = extract_wsl_distro_name(&path);
        push_file_analyze_only(
            out,
            DiskCategory::Storage,
            &format!("WSL Distro ({distro})"),
            &path,
            DiskSafety::Caution,
            "WSL virtual machine disk. Do NOT delete — this destroys the entire distro.",
        );
    }
}

#[cfg(target_os = "windows")]
fn extract_wsl_distro_name(vhdx_path: &str) -> String {
    // Path pattern: ...Packages\CanonicalGroupLimited.Ubuntu22.04LTS_79rhkp1fndgsc\LocalState\ext4.vhdx
    // We want "Ubuntu22.04LTS", not "Ubuntu22"
    let parts: Vec<&str> = vhdx_path.split('\\').collect();
    for part in &parts {
        if part.starts_with("CanonicalGroupLimited.") || part.starts_with("Microsoft.") {
            // Remove common prefixes
            let name = part
                .strip_prefix("CanonicalGroupLimited.")
                .or_else(|| part.strip_prefix("Microsoft."))
                .unwrap_or(part);
            // Split on the last underscore to remove the package hash
            // e.g. "Ubuntu22.04LTS_79rhkp1fndgsc" → "Ubuntu22.04LTS"
            if let Some(last_underscore) = name.rfind('_') {
                let candidate = &name[..last_underscore];
                if !candidate.is_empty() {
                    return candidate.to_string();
                }
            }
            return name.to_string();
        }
    }
    "Unknown".to_string()
}

// ═══════════════════════════════════════════════════════════════
//  macOS
// ═══════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
fn macos_targets(out: &mut Vec<ScanTarget>) {
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // ── System ──────────────────────────────────────────────
    push_dir(out, DiskCategory::System, "User Caches", &format!("{home}/Library/Caches"), DiskSafety::Caution, "App cache folder. Apps rebuild caches as needed.", &[]);
    push_dir(out, DiskCategory::System, "User Logs", &format!("{home}/Library/Logs"), DiskSafety::Safe, "Application log files.", &[]);
    push_dir(out, DiskCategory::System, "Cocoa Cache", &format!("{home}/Library/Caches/com.apple.Cocoa"), DiskSafety::Safe, "Cocoa framework cache.", &[]);
    push_dir(out, DiskCategory::System, "CoreServices Cache", &format!("{home}/Library/Caches/com.apple.CoreServices"), DiskSafety::Safe, "macOS CoreServices cache.", &[]);
    push_dir(out, DiskCategory::System, "System Diagnostics", &format!("{home}/Library/Logs/DiagnosticReports"), DiskSafety::Safe, "System diagnostic and crash reports.", &[]);

    // ── Browsers ────────────────────────────────────────────
    push_dir(out, DiskCategory::Browsers, "Safari Cache", &format!("{home}/Library/Caches/com.apple.Safari"), DiskSafety::Safe, "Safari web cache.", &["Safari"]);
    push_dir(out, DiskCategory::Browsers, "Safari SafeBrowsing", &format!("{home}/Library/Caches/com.apple.Safari/SafeBrowsing"), DiskSafety::Safe, "Safari safe browsing data cache.", &["Safari"]);
    push_dir(out, DiskCategory::Browsers, "Chrome Cache", &format!("{home}/Library/Caches/Google/Chrome"), DiskSafety::Safe, "Chrome browser cache.", &["Google Chrome"]);
    push_dir(out, DiskCategory::Browsers, "Chrome Code Cache", &format!("{home}/Library/Caches/Google/Chrome/Code Cache"), DiskSafety::Safe, "Chrome code cache.", &["Google Chrome"]);
    push_dir(out, DiskCategory::Browsers, "Edge Cache", &format!("{home}/Library/Caches/Microsoft Edge"), DiskSafety::Safe, "Edge browser cache.", &["Microsoft Edge"]);
    push_dir(out, DiskCategory::Browsers, "Firefox Cache", &format!("{home}/Library/Caches/Firefox"), DiskSafety::Safe, "Firefox browser cache.", &["firefox"]);
    push_dir(out, DiskCategory::Browsers, "Brave Cache", &format!("{home}/Library/Caches/BraveSoftware"), DiskSafety::Safe, "Brave browser cache.", &["Brave Browser"]);
    push_dir(out, DiskCategory::Browsers, "Opera Cache", &format!("{home}/Library/Caches/com.operasoftware.Opera"), DiskSafety::Safe, "Opera browser cache.", &["Opera"]);
    push_dir(out, DiskCategory::Browsers, "Vivaldi Cache", &format!("{home}/Library/Caches/Vivaldi"), DiskSafety::Safe, "Vivaldi browser cache.", &["Vivaldi"]);

    // ── Development — Package managers ───────────────────────
    push_dir(out, DiskCategory::Development, "npm Cache", &format!("{home}/.npm/_cacache"), DiskSafety::Safe, "npm download cache.", &[]);
    push_dir(out, DiskCategory::Development, "pnpm Cache", &format!("{home}/Library/pnpm/store"), DiskSafety::Safe, "pnpm content-addressable store.", &[]);
    push_dir(out, DiskCategory::Development, "Yarn Cache", &format!("{home}/Library/Caches/Yarn"), DiskSafety::Safe, "Yarn package cache.", &[]);
    push_dir(out, DiskCategory::Development, "pip Cache", &format!("{home}/Library/Caches/pip"), DiskSafety::Safe, "Python pip cache.", &[]);
    push_dir(out, DiskCategory::Development, "Cargo Cache", &format!("{home}/.cargo/registry/cache"), DiskSafety::Safe, "Rust crate download cache.", &[]);
    push_dir(out, DiskCategory::Development, "Cargo Git Cache", &format!("{home}/.cargo/git/db"), DiskSafety::Safe, "Rust cargo git checkout cache.", &[]);
    push_dir(out, DiskCategory::Development, "Gradle Cache", &format!("{home}/.gradle/caches"), DiskSafety::Caution, "Gradle build cache. Next build may re-download many GB.", &[]);
    push_dir(out, DiskCategory::Development, "Go Build Cache", &format!("{home}/Library/Caches/go-build"), DiskSafety::Safe, "Go build cache.", &[]);
    push_dir(out, DiskCategory::Development, "Homebrew Cache", &format!("{home}/Library/Caches/Homebrew"), DiskSafety::Safe, "Homebrew download cache.", &[]);
    push_dir(out, DiskCategory::Development, "Bun Cache", &format!("{home}/.bun/install/cache"), DiskSafety::Safe, "Bun package manager cache.", &[]);
    push_dir(out, DiskCategory::Development, "Conan Cache", &format!("{home}/.conan"), DiskSafety::Safe, "Conan C++ package manager cache.", &[]);

    // ── Development — IDEs ──────────────────────────────────
    push_dir(out, DiskCategory::Development, "VS Code Cache", &format!("{home}/Library/Application Support/Code/Cache"), DiskSafety::Safe, "VS Code cache files.", &["Code"]);
    push_dir(out, DiskCategory::Development, "VS Code CachedData", &format!("{home}/Library/Application Support/Code/CachedData"), DiskSafety::Safe, "VS Code cached editor data.", &["Code"]);
    push_dir(out, DiskCategory::Development, "VS Code GPUCache", &format!("{home}/Library/Application Support/Code/GPUCache"), DiskSafety::Safe, "VS Code GPU shader cache.", &["Code"]);
    push_dir(out, DiskCategory::Development, "VS Code Extensions Cache", &format!("{home}/Library/Application Support/Code/Service Worker/CacheStorage"), DiskSafety::Safe, "VS Code service worker cache.", &["Code"]);
    push_dir(out, DiskCategory::Development, "Cursor Cache", &format!("{home}/Library/Application Support/Cursor/Cache"), DiskSafety::Safe, "Cursor editor cache.", &["Cursor"]);
    push_dir(out, DiskCategory::Development, "Cursor CachedData", &format!("{home}/Library/Application Support/Cursor/CachedData"), DiskSafety::Safe, "Cursor cached editor data.", &["Cursor"]);
    push_dir(out, DiskCategory::Development, "JetBrains Caches", &format!("{home}/Library/Caches/JetBrains"), DiskSafety::Safe, "JetBrains IDE caches.", &[]);
    push_dir(out, DiskCategory::Development, "Xcode DerivedData", &format!("{home}/Library/Developer/Xcode/DerivedData"), DiskSafety::Caution, "Xcode build artifacts. Projects rebuild on next compile.", &["Xcode"]);
    push_dir(out, DiskCategory::Development, "Android SDK Cache", &format!("{home}/Library/Android/sdk/.temp"), DiskSafety::Safe, "Android SDK temporary build files.", &[]);

    // ── Development — Game engines ──────────────────────────
    push_dir(out, DiskCategory::Development, "Unity Cache", &format!("{home}/Library/Cache/Unity"), DiskSafety::Safe, "Unity cache. Regenerated automatically.", &["Unity"]);

    // ── Applications (dynamic) ──────────────────────────────
    discover_macos_app_caches(out, &home);

    // ── Other ───────────────────────────────────────────────
    push_dir(out, DiskCategory::Other, "QuickLook Cache", &format!("{home}/Library/Caches/com.apple.QuickLook"), DiskSafety::Safe, "Preview thumbnail cache.", &[]);
    push_dir(out, DiskCategory::Other, "Font Cache", &format!("{home}/Library/Caches/com.apple.ATS"), DiskSafety::Safe, "Font rendering cache.", &[]);
    push_dir(out, DiskCategory::Other, "Spotlight Cache", &format!("{home}/Library/Caches/com.apple.Spotlight"), DiskSafety::Safe, "Spotlight search index cache.", &[]);
    push_dir(out, DiskCategory::Other, "UIServer Cache", &format!("{home}/Library/Caches/com.apple.WindowManager"), DiskSafety::Safe, "Window Manager cache.", &[]);
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
                        | "pip" | "go-build" | "JetBrains"
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
                    &[],
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
                        | "Code" | "Cursor" | "discord" | "Spotify"
                        | "Notion" | "Figma" | "1Password" | "Microsoft"
                        | "JetBrains"
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
                        &[],
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
    locks_process: &[&str],
) {
    if path.is_empty() {
        return;
    }

    out.push(ScanTarget {
        category,
        name: name.to_string(),
        path: path.to_string(),
        kind: ScanTargetKind::Directory,
        safety,
        safety_reason: reason.to_string(),
        locks_process: locks_process.iter().map(|s| s.to_string()).collect(),
        action: CleanupAction::Delete,
    });
}

fn push_file(
    out: &mut Vec<ScanTarget>,
    category: DiskCategory,
    name: &str,
    path: &str,
    safety: DiskSafety,
    reason: &str,
    locks_process: &[&str],
) {
    if path.is_empty() {
        return;
    }

    out.push(ScanTarget {
        category,
        name: name.to_string(),
        path: path.to_string(),
        kind: ScanTargetKind::File,
        safety,
        safety_reason: reason.to_string(),
        locks_process: locks_process.iter().map(|s| s.to_string()).collect(),
        action: CleanupAction::Delete,
    });
}

fn push_file_analyze_only(
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
        kind: ScanTargetKind::File,
        safety,
        safety_reason: reason.to_string(),
        locks_process: vec![],
        action: CleanupAction::AnalyzeOnly,
    });
}

fn push_pattern(
    out: &mut Vec<ScanTarget>,
    category: DiskCategory,
    name: &str,
    parent: &str,
    pattern: &str,
    safety: DiskSafety,
    reason: &str,
) {
    if parent.is_empty() {
        return;
    }

    out.push(ScanTarget {
        category,
        name: name.to_string(),
        path: format!("{parent}\\{pattern}"),
        kind: ScanTargetKind::Pattern {
            parent: parent.to_string(),
            pattern: pattern.to_string(),
        },
        safety,
        safety_reason: reason.to_string(),
        locks_process: vec![],
        action: CleanupAction::Delete,
    });
}

fn push_virtual(
    out: &mut Vec<ScanTarget>,
    category: DiskCategory,
    name: &str,
    safety: DiskSafety,
    reason: &str,
    virtual_type: VirtualTarget,
) {
    out.push(ScanTarget {
        category,
        name: name.to_string(),
        path: format!("virtual:{:?}", virtual_type),
        kind: ScanTargetKind::Virtual { virtual_type },
        safety,
        safety_reason: reason.to_string(),
        locks_process: vec![],
        action: CleanupAction::Delete,
    });
}

/// Determines safety level for a cache subfolder name within discovered apps.
/// Some folders that look like cache actually contain state or user data.
fn cache_safety_and_reason(name: &str) -> (DiskSafety, &'static str) {
    match name {
        "Cache" | "GPUCache" | "CachedData" | "Code Cache" | "ShaderCache"
        | "DerivedDataCache" | "WebCache"
        | "CachedExtensions" | "CachedExtensionVSIXs" => {
            (DiskSafety::Safe, "Discovered application cache. Safe to clean while the app is closed.")
        }
        "logs" => {
            (DiskSafety::Safe, "Application log files. Safe to clean.")
        }
        "Crashpad" | "CrashReports" => {
            (DiskSafety::Caution, "Crash report data. May be useful for debugging. Safe to clean if you don't need crash reports.")
        }
        _ => {
            (DiskSafety::Caution, "Discovered application data. Review before deleting.")
        }
    }
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
