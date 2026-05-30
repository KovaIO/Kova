use crate::clipboard::{get_installed_apps, models::InstalledApp};

#[tauri::command]
pub fn get_apps() -> Vec<InstalledApp> {
    get_installed_apps()
}
