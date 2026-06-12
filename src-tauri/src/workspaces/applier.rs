use crate::{
    windows::manager::{resolve_rect, WindowRect},
    workspaces::{matcher::MatchedWindow, WorkspaceApp},
};
use std::collections::HashSet;
use tauri::AppHandle;

pub fn apply_window(
    app_handle: &AppHandle,
    workspace_app: &WorkspaceApp,
    matched: &MatchedWindow,
    gap: u32,
) {
    let Some(rect) = resolve_rect(
        app_handle,
        workspace_app.x,
        workspace_app.y,
        workspace_app.width,
        workspace_app.height,
        gap,
    ) else {
        return;
    };

    #[cfg(target_os = "windows")]
    move_window(matched.handle, &rect);

    #[cfg(target_os = "macos")]
    move_window(&workspace_app.name, &rect);
}

pub fn focus_window(_handle: usize, _app_name: &str) {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::{
            Foundation::HWND,
            System::Threading::{AttachThreadInput, GetCurrentThreadId},
            UI::WindowsAndMessaging::{
                BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId,
                SetForegroundWindow, ShowWindow, SW_RESTORE,
            },
        };

        unsafe {
            let hwnd = HWND(_handle as *mut core::ffi::c_void);
            let _ = ShowWindow(hwnd, SW_RESTORE);

            let foreground = GetForegroundWindow();
            let mut fg_pid = 0u32;
            let fg_thread = GetWindowThreadProcessId(foreground, Some(&mut fg_pid));
            let mut target_pid = 0u32;
            let target_thread = GetWindowThreadProcessId(hwnd, Some(&mut target_pid));
            let current_thread = GetCurrentThreadId();

            let attached_fg = fg_thread != current_thread
                && AttachThreadInput(current_thread, fg_thread, true).as_bool();
            let attached_target = target_thread != current_thread
                && target_thread != fg_thread
                && AttachThreadInput(current_thread, target_thread, true).as_bool();

            BringWindowToTop(hwnd).ok();
            let _ = SetForegroundWindow(hwnd).ok();

            if attached_fg {
                let _ = AttachThreadInput(current_thread, fg_thread, false).ok();
            }
            if attached_target {
                let _ = AttachThreadInput(current_thread, target_thread, false).ok();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
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
        System::Threading::{AttachThreadInput, GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowInfo, GetWindowThreadProcessId,
            SetForegroundWindow, SetWindowPos, ShowWindow, SWP_FRAMECHANGED, SWP_NOZORDER,
            SWP_SHOWWINDOW, SW_RESTORE, WINDOWINFO,
        },
    };

    unsafe {
        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        let _ = ShowWindow(hwnd, SW_RESTORE);

        let mut wi = WINDOWINFO {
            cbSize: std::mem::size_of::<WINDOWINFO>() as u32,
            ..Default::default()
        };
        let border = if GetWindowInfo(hwnd, &mut wi).is_ok() {
            wi.cxWindowBorders as i32
        } else {
            0
        };

        let x = rect.x as i32 - border;
        let y = rect.y as i32;
        let w = rect.width as i32 + border * 2;
        let h = rect.height as i32 + border;

        let flags = SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW;
        SetWindowPos(hwnd, None, x, y, w, h, flags).ok();
        std::thread::sleep(std::time::Duration::from_millis(50));
        SetWindowPos(hwnd, None, x, y, w, h, flags).ok();

        let foreground = GetForegroundWindow();
        let mut fg_pid = 0u32;
        let fg_thread = GetWindowThreadProcessId(foreground, Some(&mut fg_pid));
        let mut target_pid = 0u32;
        let target_thread = GetWindowThreadProcessId(hwnd, Some(&mut target_pid));
        let current_thread = GetCurrentThreadId();

        let attached_fg = fg_thread != current_thread
            && AttachThreadInput(current_thread, fg_thread, true).as_bool();
        let attached_target = target_thread != current_thread
            && target_thread != fg_thread
            && AttachThreadInput(current_thread, target_thread, true).as_bool();

        BringWindowToTop(hwnd).ok();
        let _ = SetForegroundWindow(hwnd).ok();

        if attached_fg {
            let _ = AttachThreadInput(current_thread, fg_thread, false).ok();
        }
        if attached_target {
            let _ = AttachThreadInput(current_thread, target_thread, false).ok();
        }
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

pub fn minimize_other_windows(keep_handles: &HashSet<usize>) {
    #[cfg(target_os = "windows")]
    {
        minimize_other_windows_windows(keep_handles);
    }

    #[cfg(target_os = "macos")]
    {
        minimize_other_windows_macos();
    }
}

#[cfg(target_os = "windows")]
fn minimize_other_windows_windows(keep_handles: &HashSet<usize>) {
    use windows::{
        core::BOOL,
        Win32::{
            Foundation::{HWND, LPARAM},
            UI::WindowsAndMessaging::{
                EnumWindows, GetClassNameW, GetWindowThreadProcessId, IsWindowVisible, ShowWindow,
                SW_MINIMIZE,
            },
        },
    };

    let current_pid = unsafe { windows::Win32::System::Threading::GetCurrentProcessId() };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &*(lparam.0 as *const EnumData);

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        // Skip windows we want to keep
        if data.keep.contains(&(hwnd.0 as usize)) {
            return true.into();
        }

        // Skip windows belonging to our own process
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == data.current_pid {
            return true.into();
        }

        // Skip taskbar and system tray
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf) as usize;
        let class = String::from_utf16_lossy(&class_buf[..class_len]);
        if class == "Shell_TrayWnd"
            || class == "Shell_SecondaryTrayWnd"
            || class == "WorkerW"
            || class == "Progman"
        {
            return true.into();
        }

        // Skip windows with no title (background/hidden windows)
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextLengthW;
        let title_len = GetWindowTextLengthW(hwnd);
        if title_len == 0 {
            return true.into();
        }

        // Skip tool windows (WS_EX_TOOLWINDOW)
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
        };
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if ex_style & WS_EX_TOOLWINDOW.0 as i32 != 0 {
            return true.into();
        }

        let _ = ShowWindow(hwnd, SW_MINIMIZE);

        true.into()
    }

    struct EnumData {
        keep: HashSet<usize>,
        current_pid: u32,
    }

    let data = EnumData {
        keep: keep_handles.clone(),
        current_pid,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&data as *const _ as isize));
    }
}

#[cfg(target_os = "macos")]
fn minimize_other_windows_macos() {
    // Minimize all windows except the frontmost app using AppleScript
    let script = r#"
        tell application "System Events"
            set visibleApps to name of every application process whose visible is true
            repeat with appName in visibleApps
                if appName is not "Kova" then
                    try
                        tell application process appName
                            set miniaturized of every window to true
                        end tell
                    end try
                end if
            end repeat
        end tell
    "#;
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
}
