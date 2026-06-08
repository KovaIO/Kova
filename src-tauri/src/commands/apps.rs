use crate::apps::{get_installed_apps, InstalledApp};

#[tauri::command]
pub fn get_apps() -> Vec<InstalledApp> {
    get_installed_apps()
}
