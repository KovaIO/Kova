use tauri::{AppHandle, Emitter, State};

use crate::{
    app_state::AppState,
    license::{LicenseInfo, LicenseTier},
};

#[tauri::command]
pub fn get_license(state: State<AppState>) -> Result<LicenseInfo, String> {
    state.license.get_info().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn activate_license(app: AppHandle, state: State<AppState>) -> Result<LicenseInfo, String> {
    let info = state
        .license
        .set_tier(LicenseTier::Pro)
        .map_err(|e| e.to_string())?;
    app.emit("license-updated", &info).ok();
    Ok(info)
}
