use crate::windows;
use tauri::AppHandle;

#[tauri::command]
pub fn open_preferences(app: AppHandle) {
    windows::open_window(&app, "prefs");
}
