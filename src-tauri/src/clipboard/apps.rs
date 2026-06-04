use std::collections::HashMap;

use crate::clipboard::models::InstalledApp;
use crate::processes::get_process_icon;

pub fn get_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut deduped: HashMap<String, InstalledApp> = HashMap::new();

    #[cfg(target_os = "macos")]
    apps.extend(get_macos_apps());

    #[cfg(target_os = "windows")]
    apps.extend(get_windows_apps());

    for app in apps {
        let key = app_key(&app);

        match deduped.get(&key) {
            Some(existing) => {
                if app_score(&app) > app_score(existing) {
                    deduped.insert(key, app);
                }
            }
            None => {
                deduped.insert(key, app);
            }
        }
    }

    let mut apps: Vec<InstalledApp> = deduped.into_values().collect();

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn app_key(app: &InstalledApp) -> String {
    if !app.path.is_empty() {
        let path = std::path::Path::new(&app.path);

        let dir = if path.extension().is_some() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        return dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("\\")
            .to_lowercase();
    }

    app.name.to_lowercase()
}

fn app_score(app: &InstalledApp) -> u32 {
    let mut score = 0;

    if app.icon.is_some() {
        score += 100;
    }

    if app.path.to_lowercase().ends_with(".exe") {
        score += 50;
    }

    if !app.path.is_empty() {
        score += 20;
    }

    score += app.name.len() as u32;

    score
}

#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
fn get_macos_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let search_dirs = [
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        dirs::home_dir()
            .map(|h| h.join("Applications"))
            .unwrap_or_default(),
    ];

    for dir in &search_dirs {
        scan_macos_dir(dir, &mut apps, 0);
    }

    apps
}

#[cfg(target_os = "macos")]
fn scan_macos_dir(dir: &PathBuf, apps: &mut Vec<InstalledApp>, depth: u32) {
    if depth > 2 {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if path.extension().map(|e| e == "app").unwrap_or(false) {
            let macos_dir = path.join("Contents/MacOS");
            if macos_dir.is_dir() {
                let path_str = path.to_string_lossy().to_string();
                let icon = get_process_icon(&name, Some(&path_str));
                apps.push(InstalledApp {
                    name,
                    path: path_str,
                    exe_path: None,
                    icon,
                });
            }
        } else if path.is_dir() {
            scan_macos_dir(&path, apps, depth + 1);
        }
    }
}

#[cfg(target_os = "windows")]
use windows::{
    core::{Interface, PCWSTR, PWSTR},
    Management::Deployment::PackageManager,
    Win32::{
        Foundation::ERROR_NO_MORE_ITEMS,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
                COINIT_APARTMENTTHREADED, STGM_READ,
            },
            Environment::ExpandEnvironmentStringsW,
            Registry::{
                RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY,
                HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
                REG_VALUE_TYPE,
            },
        },
        UI::Shell::{IShellLinkW, ShellLink},
    },
};

#[cfg(target_os = "windows")]
fn get_windows_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let uninstall_paths: &[(HKEY, &str)] = &[
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (hive, subkey_path) in uninstall_paths {
        let wide: Vec<u16> = subkey_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut hkey = HKEY::default();

        if unsafe {
            RegOpenKeyExW(
                *hive,
                PCWSTR(wide.as_ptr()),
                Some(0u32),
                KEY_READ,
                &mut hkey,
            )
        }
        .is_err()
        {
            continue;
        }

        let mut index = 0u32;
        loop {
            let mut name_buf = vec![0u16; 1024];
            let mut name_len = name_buf.len() as u32;

            let res = unsafe {
                RegEnumKeyExW(
                    hkey,
                    index,
                    Some(PWSTR(name_buf.as_mut_ptr())),
                    &mut name_len,
                    None,
                    None,
                    None,
                    None,
                )
            };

            if res == ERROR_NO_MORE_ITEMS.into() {
                break;
            }
            if res.is_err() {
                index += 1;
                continue;
            }

            let subkey_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let full_path = format!(r"{}\{}", subkey_path, subkey_name);
            let full_wide: Vec<u16> = full_path.encode_utf16().chain(std::iter::once(0)).collect();

            let mut subkey = HKEY::default();
            if unsafe {
                RegOpenKeyExW(
                    *hive,
                    PCWSTR(full_wide.as_ptr()),
                    Some(0u32),
                    KEY_READ,
                    &mut subkey,
                )
            }
            .is_ok()
            {
                if let Some(app) = parse_uninstall_entry(subkey) {
                    apps.push(app);
                }
                let _ = unsafe { RegCloseKey(subkey) };
            }

            index += 1;
        }

        let _ = unsafe { RegCloseKey(hkey) };
    }

    apps.extend(get_app_paths_apps());
    apps.extend(get_start_menu_apps());
    apps.extend(get_store_apps());

    apps
}

#[cfg(target_os = "windows")]
fn parse_uninstall_entry(subkey: HKEY) -> Option<InstalledApp> {
    let name = read_reg_string(subkey, "DisplayName")?;

    let is_component = read_reg_dword(subkey, "SystemComponent")
        .map(|v| v == 1)
        .unwrap_or(false);
    if is_component {
        return None;
    }

    let name_lower = name.to_lowercase();
    if name_lower.starts_with("kb") || name_lower.contains("windows update") {
        return None;
    }

    let icon_source = parse_icon_source(read_reg_string(subkey, "DisplayIcon"));

    let install_location = read_reg_string(subkey, "InstallLocation")
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| std::path::Path::new(s).is_dir());

    if name.trim().is_empty() {
        return None;
    }

    let check = icon_source
        .as_deref()
        .or(install_location.as_deref())
        .unwrap_or("")
        .to_lowercase();

    if check.contains(r"windows\system32")
        || check.contains(r"windows\syswow64")
        || check.contains(r"windows\winsxs")
    {
        return None;
    }

    let stored_path = install_location.clone().unwrap_or_else(|| {
        icon_source
            .as_deref()
            .and_then(|p| std::path::Path::new(p).parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let icon = get_process_icon(&name, icon_source.as_deref());

    let exe_path = icon_source
        .as_ref()
        .filter(|p| p.to_lowercase().ends_with(".exe"))
        .cloned();

    Some(InstalledApp {
        name,
        exe_path,
        path: stored_path,
        icon,
    })
}

#[cfg(target_os = "windows")]
fn get_app_paths_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let paths: &[(HKEY, &str)] = &[
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
        ),
    ];

    for (hive, subkey_path) in paths {
        let wide: Vec<u16> = subkey_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut hkey = HKEY::default();

        if unsafe {
            RegOpenKeyExW(
                *hive,
                PCWSTR(wide.as_ptr()),
                Some(0u32),
                KEY_READ,
                &mut hkey,
            )
        }
        .is_err()
        {
            continue;
        }

        let mut index = 0u32;
        loop {
            let mut name_buf = vec![0u16; 1024];
            let mut name_len = name_buf.len() as u32;

            let res = unsafe {
                RegEnumKeyExW(
                    hkey,
                    index,
                    Some(PWSTR(name_buf.as_mut_ptr())),
                    &mut name_len,
                    None,
                    None,
                    None,
                    None,
                )
            };

            if res == ERROR_NO_MORE_ITEMS.into() {
                break;
            }
            if res.is_err() {
                index += 1;
                continue;
            }

            let entry_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let full_path = format!(r"{}\{}", subkey_path, entry_name);
            let full_wide: Vec<u16> = full_path.encode_utf16().chain(std::iter::once(0)).collect();

            let mut subkey = HKEY::default();
            if unsafe {
                RegOpenKeyExW(
                    *hive,
                    PCWSTR(full_wide.as_ptr()),
                    Some(0u32),
                    KEY_READ,
                    &mut subkey,
                )
            }
            .is_ok()
            {
                if let Some(raw) = read_reg_string(subkey, "") {
                    let exe_path = raw.trim().trim_matches('"').to_string();
                    let p = std::path::Path::new(&exe_path);

                    if p.exists() && p.extension().map(|e| e == "exe").unwrap_or(false) {
                        let path_lower = exe_path.to_lowercase();
                        if !path_lower.contains(r"windows\system32")
                            && !path_lower.contains(r"windows\syswow64")
                        {
                            let raw_stem = std::path::Path::new(&entry_name)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();

                            let name = if raw_stem.chars().all(|c| c.is_uppercase() || c == '_') {
                                raw_stem
                                    .chars()
                                    .enumerate()
                                    .map(|(i, c)| {
                                        if i == 0 {
                                            c.to_uppercase().next().unwrap_or(c)
                                        } else {
                                            c.to_lowercase().next().unwrap_or(c)
                                        }
                                    })
                                    .collect()
                            } else {
                                raw_stem
                            };

                            let stored_path = p
                                .parent()
                                .map(|d| d.to_string_lossy().to_string())
                                .unwrap_or_else(|| exe_path.clone());

                            let icon = get_process_icon(&name, Some(&exe_path));

                            apps.push(InstalledApp {
                                name,
                                exe_path: Some(exe_path.clone()),
                                path: stored_path,
                                icon,
                            });
                        }
                    }
                }
                let _ = unsafe { RegCloseKey(subkey) };
            }

            index += 1;
        }

        let _ = unsafe { RegCloseKey(hkey) };
    }

    apps
}

#[cfg(target_os = "windows")]
fn expand_env_vars(value: &str) -> String {
    let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buffer = vec![0u16; 4096];

    let written = unsafe { ExpandEnvironmentStringsW(PCWSTR(wide.as_ptr()), Some(&mut buffer)) };

    if written == 0 {
        return value.to_string();
    }

    let len = written.saturating_sub(1) as usize;
    String::from_utf16_lossy(&buffer[..len])
}

#[cfg(target_os = "windows")]
fn read_reg_string(key: HKEY, value: &str) -> Option<String> {
    let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let mut data_type = REG_VALUE_TYPE::default();
    let mut size = 0u32;

    unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(wide.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut size),
        )
    }
    .ok()
    .ok()?;

    if data_type != REG_SZ && data_type != REG_EXPAND_SZ {
        return None;
    }

    let mut buf = vec![0u16; size as usize / 2 + 1];
    unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(wide.as_ptr()),
            None,
            None,
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut size),
        )
    }
    .ok()
    .ok()?;

    while buf.last() == Some(&0) {
        buf.pop();
    }

    if data_type == REG_EXPAND_SZ {
        return Some(expand_env_vars(&String::from_utf16_lossy(&buf)));
    }

    Some(String::from_utf16_lossy(&buf))
}

#[cfg(target_os = "windows")]
fn read_reg_dword(key: HKEY, value: &str) -> Option<u32> {
    let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let mut data_type = REG_VALUE_TYPE::default();
    let mut data = 0u32;
    let mut size = 4u32;

    unsafe {
        RegQueryValueExW(
            key,
            PCWSTR(wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut size),
        )
    }
    .ok()
    .ok()?;

    (data_type == REG_DWORD).then_some(data)
}

#[cfg(target_os = "windows")]
fn parse_icon_source(icon: Option<String>) -> Option<String> {
    let icon = icon?;
    let path_str = icon.split(',').next()?.trim().trim_matches('"');
    let expanded = expand_env_vars(path_str);
    let path = std::path::Path::new(&expanded);

    if !path.exists() {
        return None;
    }

    let ext = path.extension()?.to_string_lossy().to_lowercase();
    if ext == "exe" || ext == "dll" || ext == "ico" {
        Some(expanded)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn get_start_menu_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let mut dirs = Vec::new();

    dirs.push(std::path::PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs",
    ));

    if let Some(roaming) = dirs::data_dir() {
        dirs.push(
            roaming
                .parent()
                .unwrap_or(&roaming)
                .join("Roaming")
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }

    for dir in dirs {
        scan_start_menu_dir(&dir, &mut apps);
    }

    apps
}

#[cfg(target_os = "windows")]
fn scan_start_menu_dir(dir: &std::path::Path, apps: &mut Vec<InstalledApp>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_start_menu_dir(&path, apps);
            continue;
        }

        if path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false)
        {
            if let Some(target) = resolve_shortcut(&path) {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let icon = get_process_icon(&name, Some(&target));

                apps.push(InstalledApp {
                    name,
                    exe_path: Some(target.clone()),
                    path: target.to_string(),
                    icon,
                });
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn resolve_shortcut(path: &std::path::Path) -> Option<String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().ok()?;

        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;

        let persist: IPersistFile = shell_link.cast().ok()?;

        let wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        persist.Load(PCWSTR(wide.as_ptr()), STGM_READ).ok()?;

        let mut buffer = [0u16; 260];

        shell_link
            .GetPath(&mut buffer, std::ptr::null_mut(), 0)
            .ok()?;

        let len = buffer.iter().position(|v| *v == 0).unwrap_or(0);

        Some(String::from_utf16_lossy(&buffer[..len]))
    }
}

#[cfg(target_os = "windows")]
fn get_store_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    let Ok(pm) = PackageManager::new() else {
        return apps;
    };

    let Ok(packages) = pm.FindPackages() else {
        return apps;
    };

    for package in packages {
        if package.IsFramework().unwrap_or(false) {
            continue;
        }

        let name = package
            .Id()
            .ok()
            .and_then(|id| id.Name().ok())
            .unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        apps.push(InstalledApp {
            name: name.to_string(),
            exe_path: None,
            path: String::new(),
            icon: None,
        });
    }

    apps
}
