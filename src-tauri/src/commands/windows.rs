use crate::windows;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn open_preferences(app: AppHandle) {
    windows::open_window(&app, "prefs");
}

#[tauri::command]
pub fn open_monitor(app: AppHandle, tab: Option<String>) {
    if let Some(window) = app.get_webview_window("monitor") {
        let tab = tab.unwrap_or_else(|| "cpu".to_string());
        if let Ok(current_url) = window.url() {
            let origin = current_url.origin().ascii_serialization();
            let url = format!("{}/monitor?tab={}", origin, tab);

            if let Ok(parsed) = url.parse() {
                let _ = window.navigate(parsed);
            }
        }
        windows::open_window(&app, "monitor");
    }
}
