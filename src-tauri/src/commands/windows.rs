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
