use base64::{engine::general_purpose, Engine};
use image::{ImageBuffer, ImageFormat, Rgba};

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{LazyLock, Mutex},
};

const MAX_ICON_CACHE: usize = 256;

static ICON_CACHE: LazyLock<Mutex<HashMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_process_icon(name: &str, exe_path: Option<&str>) -> Option<String> {
    let Some(path) = exe_path else {
        return fallback_icon(name, None);
    };

    {
        let cache = ICON_CACHE.lock().unwrap();

        if let Some(icon) = cache.get(path) {
            return icon.clone();
        }
    }

    let icon = platform::get_process_icon_base64(path).or_else(|| fallback_icon(name, Some(path)));

    let mut cache = ICON_CACHE.lock().unwrap();

    if cache.len() >= MAX_ICON_CACHE {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }

    cache.insert(path.to_string(), icon.clone());

    icon
}

pub fn get_app_icon(
    name: &str,
    exe_path: Option<&str>,
    install_path: Option<&str>,
) -> Option<String> {
    let cache_key = exe_path.or(install_path)?;

    {
        let cache = ICON_CACHE.lock().unwrap();
        if let Some(icon) = cache.get(cache_key) {
            return icon.clone();
        }
    }

    let derived_dir;
    let effective_install = if install_path.is_some() {
        install_path
    } else {
        derived_dir = exe_path
            .and_then(|p| std::path::Path::new(p).parent())
            .map(|p| p.to_string_lossy().into_owned());
        derived_dir.as_deref()
    };

    let icon = platform::get_app_icon_base64(name, exe_path, effective_install)
        .or_else(|| fallback_icon(name, exe_path));

    let mut cache = ICON_CACHE.lock().unwrap();
    if cache.len() >= MAX_ICON_CACHE {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }
    cache.insert(cache_key.to_string(), icon.clone());

    icon
}
fn fallback_icon(name: &str, exe_path: Option<&str>) -> Option<String> {
    let lower = name.to_lowercase();

    let is_system = lower.contains("system")
        || lower.contains("service")
        || lower.contains("registry")
        || lower.contains("svchost")
        || exe_path.is_none();

    if is_system {
        Some("system".to_string())
    } else {
        Some("terminal".to_string())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PCWSTR;

    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS,
    };

    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;

    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};

    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

    pub fn get_process_icon_base64(path: &str) -> Option<String> {
        unsafe {
            let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();

            let mut file_info = SHFILEINFOW::default();

            let result = SHGetFileInfoW(
                PCWSTR(wide.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut file_info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );

            if result == 0 {
                return None;
            }

            let icon = file_info.hIcon;

            let png = icon_to_png(icon)?;

            let _ = DestroyIcon(icon);

            Some(general_purpose::STANDARD.encode(png))
        }
    }

    pub fn get_app_icon_base64(
        name: &str,
        exe_path: Option<&str>,
        install_path: Option<&str>,
    ) -> Option<String> {
        if let Some(path) = exe_path {
            let stem = std::path::Path::new(path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();

            let is_launcher = [
                "update",
                "uninstall",
                "setup",
                "installer",
                "helper",
                "crashpad",
            ]
            .iter()
            .any(|n| stem.contains(n));

            if !is_launcher {
                if let Some(icon) = get_process_icon_base64(path) {
                    return Some(icon);
                }
            }
        }

        if let Some(dir) = install_path {
            if let Some(icon) = find_best_exe_icon(name, dir) {
                return Some(icon);
            }
        }

        None
    }

    unsafe fn icon_to_png(icon: HICON) -> Option<Vec<u8>> {
        let mut icon_info = ICONINFO::default();

        if GetIconInfo(icon, &mut icon_info).is_err() {
            return None;
        }

        let mut bmp = BITMAP::default();

        if GetObjectW(
            icon_info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut _),
        ) == 0
        {
            return None;
        }

        let width = bmp.bmWidth as u32;
        let height = bmp.bmHeight as u32;

        let hdc = CreateCompatibleDC(None);

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let result = GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        let _ = DeleteDC(hdc);

        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());

        if result == 0 {
            return None;
        }

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, pixels)?;

        let mut png = Vec::new();

        image
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .ok()?;

        Some(png)
    }

    fn find_best_exe_icon(app_name: &str, install_dir: &str) -> Option<String> {
        find_best_exe_icon_depth(app_name, install_dir, 0)
    }

    fn find_best_exe_icon_depth(app_name: &str, install_dir: &str, depth: u32) -> Option<String> {
        use std::path::Path;

        let dir = Path::new(install_dir);
        if !dir.is_dir() {
            return None;
        }

        let name_lower = app_name.to_lowercase();

        let Ok(entries) = std::fs::read_dir(dir) else {
            return None;
        };

        let entries: Vec<_> = entries.flatten().collect();

        let mut candidates: Vec<(u32, std::path::PathBuf)> = entries
            .iter()
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_string_lossy().to_lowercase() == "exe" {
                    let stem = p.file_stem()?.to_string_lossy().to_lowercase();
                    let score = score_exe_candidate(&stem, &name_lower);
                    Some((score, p))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        for (score, path) in &candidates {
            if *score == 0 {
                break;
            }
            let path_str = path.to_string_lossy();
            if let Some(icon) = get_process_icon_base64(&path_str) {
                return Some(icon);
            }
        }

        if depth < 1 {
            let mut subdirs: Vec<_> = entries
                .iter()
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();

            subdirs.sort_by_key(|p| {
                let n = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                if n.contains(&name_lower) {
                    0u8
                } else {
                    1u8
                }
            });

            for subdir in subdirs {
                let subdir_str = subdir.to_string_lossy();
                if let Some(icon) = find_best_exe_icon_depth(app_name, &subdir_str, depth + 1) {
                    return Some(icon);
                }
            }
        }

        None
    }

    fn score_exe_candidate(stem: &str, app_name: &str) -> u32 {
        if stem == app_name {
            return 100;
        }
        if stem.contains(app_name) || app_name.contains(stem) {
            return 80;
        }
        let words: Vec<&str> = app_name.split_whitespace().collect();
        let matching_words = words
            .iter()
            .filter(|w| w.len() > 3 && stem.contains(**w))
            .count();
        if matching_words > 0 {
            return 60 + (matching_words as u32 * 10);
        }
        let first_word = words.first().copied().unwrap_or("");
        if first_word.len() > 3 && stem.contains(first_word) {
            return 60;
        }
        let noise = [
            "uninstall",
            "update",
            "helper",
            "crash",
            "setup",
            "installer",
            "crashpad",
        ];
        if noise.iter().any(|n| stem.contains(n)) {
            return 0;
        }
        10
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;

    use objc::class;
    use objc::{msg_send, sel, sel_impl};

    pub fn get_process_icon_base64(path: &str) -> Option<String> {
        unsafe {
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];

            let ns_path = NSString::alloc(nil).init_str(path);

            let icon: id = msg_send![workspace, iconForFile: ns_path];

            if icon == nil {
                return None;
            }

            let tiff_data: id = msg_send![icon, TIFFRepresentation];

            if tiff_data == nil {
                return None;
            }

            let bytes: *const u8 = msg_send![tiff_data, bytes];

            let len: usize = msg_send![tiff_data, length];

            let slice = std::slice::from_raw_parts(bytes, len);

            Some(general_purpose::STANDARD.encode(slice))
        }
    }

    pub fn get_app_icon_base64(
        _name: &str,
        exe_path: Option<&str>,
        _install_path: Option<&str>,
    ) -> Option<String> {
        exe_path.and_then(|p| get_process_icon_base64(p))
    }
}
