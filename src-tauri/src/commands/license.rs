use tauri::{AppHandle, Emitter, State};

use crate::{app_state::AppState, license::LicenseInfo};

#[tauri::command]
pub fn get_license(state: State<AppState>) -> Result<LicenseInfo, String> {
    state.license.get_info().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn activate_license(
    app: AppHandle,
    state: State<'_, AppState>,
    email: String,
) -> Result<LicenseInfo, String> {
    let device_name = whoami::devicename().map_err(|e| e.to_string())?;

    let platform = std::env::consts::OS.to_string();

    let info = state
        .license
        .activate_license(email, device_name, platform)
        .await?;

    app.emit("license-updated", &info).ok();

    Ok(info)
}
