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
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    pub fn get_process_icon_base64(_path: &str) -> Option<String> {
        None
    }
}
