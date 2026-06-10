use crate::{app_state::AppState, license::LicenseTier, workspaces::WorkspaceProfile};
use tauri::State;

#[tauri::command]
pub fn get_workspace_profiles(state: State<AppState>) -> Result<Vec<WorkspaceProfile>, String> {
    state.workspaces.get_profiles().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_workspace_profile(
    profile_id: String,
    state: State<AppState>,
) -> Result<Option<WorkspaceProfile>, String> {
    state
        .workspaces
        .get_profile(&profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_workspace_profile(
    profile: WorkspaceProfile,
    state: State<AppState>,
) -> Result<(), String> {
    let tier = state.license.tier().map_err(|e| e.to_string())?;
    if tier != LicenseTier::Pro {
        return Err("Workspace profiles require a Pro license".into());
    }
    state
        .workspaces
        .save_profile(profile)
        .map_err(|e| e.to_string())?;
    state.emit_workspaces_updated();
    Ok(())
}

#[tauri::command]
pub fn delete_workspace_profile(profile_id: String, state: State<AppState>) -> Result<(), String> {
    state
        .workspaces
        .delete_profile(&profile_id)
        .map_err(|e| e.to_string())?;
    state.emit_workspaces_updated();
    Ok(())
}

#[tauri::command]
pub async fn apply_workspace(profile_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let tier = state.license.tier().map_err(|e| e.to_string())?;
    if tier != LicenseTier::Pro {
        return Err("Applying workspaces requires a Pro license".into());
    }
    state
        .workspaces
        .apply_profile(&profile_id, &state.app_handle)
        .await
}
