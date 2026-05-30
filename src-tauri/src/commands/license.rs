use tauri::State;

use crate::{app_state::AppState, license::LicenseInfo};

#[tauri::command]
pub fn get_license(state: State<AppState>) -> Result<LicenseInfo, String> {
    state.license.get_info().map_err(|e| e.to_string())
}
