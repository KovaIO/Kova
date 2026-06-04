use crate::{
    windows::manager::{resolve_rect, WindowRect},
    workspaces::{matcher::MatchedWindow, WorkspaceApp},
};
use tauri::AppHandle;

pub fn apply_window(app_handle: &AppHandle, workspace_app: &WorkspaceApp, matched: &MatchedWindow) {
    let Some(rect) = resolve_rect(
        app_handle,
        workspace_app.x,
        workspace_app.y,
        workspace_app.width,
        workspace_app.height,
    ) else {
        return;
    };

    #[cfg(target_os = "windows")]
    move_window(matched.handle, &rect);

    #[cfg(target_os = "macos")]
    move_window(&workspace_app.name, &rect);
}

pub fn focus_window(handle: usize, _app_name: &str) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{
                BringWindowToTop, SetForegroundWindow, ShowWindow, SW_RESTORE,
            },
        };
        unsafe {
            let hwnd = HWND(handle as *mut core::ffi::c_void);
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = BringWindowToTop(hwnd);
            let _ = SetForegroundWindow(hwnd);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = handle;
        let script = format!(r#"tell application "{}" to activate"#, _app_name);
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    }
}

#[cfg(target_os = "windows")]
pub fn move_window(hwnd: usize, rect: &WindowRect) {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            BringWindowToTop, SetForegroundWindow, SetWindowPos, ShowWindow, SWP_FRAMECHANGED,
            SWP_NOZORDER, SWP_SHOWWINDOW, SW_RESTORE,
        },
    };
    unsafe {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let x = rect.x as i32;
        let y = rect.y as i32;
        let w = rect.width as i32;
        let h = rect.height as i32;
        let flags = SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW;
        let _ = SetWindowPos(hwnd, None, x, y, w, h, flags);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let _ = SetWindowPos(hwnd, None, x, y, w, h, flags);
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
    }
}

#[cfg(target_os = "macos")]
pub fn move_window(app_name: &str, rect: &WindowRect) {
    let script = format!(
        r#"tell application "{app}"
            activate
            delay 0.2
            set bounds of front window to {{{x}, {y}, {r}, {b}}}
        end tell"#,
        app = app_name,
        x = rect.x as i64,
        y = rect.y as i64,
        r = (rect.x + rect.width) as i64,
        b = (rect.y + rect.height) as i64,
    );

    let result = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();

    if let Err(e) = result {
        println!("AppleScript failed for '{}': {}", app_name, e);
    }
}
