use tauri::{AppHandle, Manager};

pub fn open_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        window.show().ok();
        window.set_focus().ok();
    }
}

// pub fn hide_window(app: &AppHandle, label: &str) {
//     if let Some(window) = app.get_webview_window(label) {
//         window.hide().ok();
//     }
// }

// pub fn toggle_window(app: &AppHandle, label: &str) {
//     if let Some(window) = app.get_webview_window(label) {
//         if window.is_visible().unwrap_or(false) {
//             window.hide().ok();
//         } else {
//             window.show().ok();
//             window.set_focus().ok();
//         }
//     }
// }
