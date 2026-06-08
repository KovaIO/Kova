use std::path::Path;

use crate::{apps::InstalledApp, clipboard::models::SourceApp};

pub fn get_foreground_app() -> Option<SourceApp> {
    #[cfg(target_os = "windows")]
    {
        return get_foreground_app_windows();
    }

    #[cfg(target_os = "macos")]
    {
        return get_foreground_app_macos();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

pub fn is_app_ignored(foreground: &SourceApp, ignored: &[InstalledApp]) -> bool {
    let exe = foreground.path.to_lowercase();

    ignored.iter().any(|app| {
        if app.path.is_empty() {
            return app.name.eq_ignore_ascii_case(&foreground.name);
        }

        let ignored_path = app.path.to_lowercase();

        if exe.is_empty() {
            return app.name.eq_ignore_ascii_case(&foreground.name);
        }

        exe.starts_with(&ignored_path)
            || Path::new(&exe)
                .parent()
                .map(|parent| {
                    parent
                        .to_string_lossy()
                        .to_lowercase()
                        .starts_with(&ignored_path)
                })
                .unwrap_or(false)
    })
}

#[cfg(target_os = "windows")]
fn get_foreground_app_windows() -> Option<SourceApp> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;

        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .ok()?;

        let _ = CloseHandle(process);

        let path = OsString::from_wide(&buffer[..size as usize])
            .to_string_lossy()
            .to_string();

        let name = Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown".to_string());

        Some(SourceApp { name, path })
    }
}

#[cfg(target_os = "macos")]
fn get_foreground_app_macos() -> Option<SourceApp> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }

        let app: id = msg_send![workspace, frontmostApplication];
        if app == nil {
            return None;
        }

        let name_ns: id = msg_send![app, localizedName];
        let path_ns: id = msg_send![app, bundlePath];

        let name = if name_ns != nil {
            nsstring_to_string(name_ns)
        } else {
            "Unknown".to_string()
        };

        let path = if path_ns != nil {
            nsstring_to_string(path_ns)
        } else {
            String::new()
        };

        Some(SourceApp { name, path })
    }
}
#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(ns: id) -> String {
    use std::ffi::CStr;

    let utf8: id = msg_send![ns, UTF8String];
    if utf8.is_null() {
        return String::new();
    }

    CStr::from_ptr(utf8 as *const i8)
        .to_string_lossy()
        .into_owned()
}
