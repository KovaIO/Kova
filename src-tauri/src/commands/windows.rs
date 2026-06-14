use crate::windows;
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub fn open_preferences(app: AppHandle) {
    windows::open_window(&app, "prefs");
}

#[tauri::command]
pub fn open_monitor(app: AppHandle, tab: Option<String>) {
    if let Some(window) = app.get_webview_window("monitor") {
        let tab = tab.unwrap_or_else(|| "cpu".to_string());
        let _ = window.emit_to("monitor", "set-tab", tab);
        windows::apply_window_density(&app, "monitor");
        windows::open_window(&app, "monitor");
    }
}

#[tauri::command]
pub fn open_process(app: AppHandle, pid: u32, tab: Option<String>) {
    if let Some(window) = app.get_webview_window("process") {
        let tab = tab.unwrap_or_else(|| "cpu".to_string());
        let _ = window.emit_to(
            "process",
            "set-process",
            serde_json::json!({ "pid": pid, "tab": tab }),
        );
        windows::apply_window_density(&app, "process");
        windows::open_window(&app, "process");
    }
}

#[tauri::command]
pub fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(".onboarding_completed");

    std::fs::write(path, b"").map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
pub fn set_menu_bar_visible(_app: AppHandle, visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let main_menu: *mut Object = msg_send![app, mainMenu];
            if !main_menu.is_null() {
                let _: () = msg_send![main_menu, setHidden: !visible];
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = _app;
        let _ = visible;
    }
    Ok(())
}
