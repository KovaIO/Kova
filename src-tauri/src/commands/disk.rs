use tauri::State;

use crate::{
    app_state::AppState,
    disk::models::{DiskItemDetail, DiskScanResult, DiskVolumeInfo, ScanPreview},
    license::limits::limits_for_tier,
};

fn ensure_disk_clean(state: &State<AppState>) -> Result<(), String> {
    let tier = state.license.tier().map_err(|e| e.to_string())?;
    if !limits_for_tier(&tier).disk_clean {
        return Err("Disk clean requires a Pro license".into());
    }
    Ok(())
}

#[tauri::command]
pub fn get_disk_volume_info(state: State<AppState>) -> Result<DiskVolumeInfo, String> {
    ensure_disk_clean(&state)?;
    Ok(state.disk.volume_info())
}

#[tauri::command]
pub fn get_disk_scan_preview(state: State<AppState>) -> Result<ScanPreview, String> {
    ensure_disk_clean(&state)?;
    Ok(state.disk.scan_preview())
}

#[tauri::command]
pub fn get_disk_scan_result(state: State<AppState>) -> Result<Option<DiskScanResult>, String> {
    ensure_disk_clean(&state)?;
    Ok(state.disk.get_scan_result())
}

#[tauri::command]
pub fn get_disk_item_detail(
    state: State<AppState>,
    item_id: String,
) -> Result<DiskItemDetail, String> {
    ensure_disk_clean(&state)?;
    state.disk.get_item_detail(&item_id)
}

#[tauri::command]
pub async fn start_disk_scan(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DiskScanResult, String> {
    ensure_disk_clean(&state)?;
    state.disk.run_scan(app).await
}

#[tauri::command]
pub fn delete_disk_items(state: State<AppState>, ids: Vec<String>) -> Result<u64, String> {
    ensure_disk_clean(&state)?;
    state.disk.delete_items(&ids)
}

#[tauri::command]
pub fn delete_all_disk_items(state: State<AppState>) -> Result<u64, String> {
    ensure_disk_clean(&state)?;

    let result = state
        .disk
        .get_scan_result()
        .ok_or_else(|| "No scan results available".to_string())?;

    let ids: Vec<String> = result
        .categories
        .iter()
        .flat_map(|group| group.items.iter())
        .filter(|item| item.safety != crate::disk::models::DiskSafety::Unsafe)
        .map(|item| item.id.clone())
        .collect();

    state.disk.delete_items(&ids)
}
