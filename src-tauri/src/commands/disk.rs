use tauri::State;

use crate::{
    app_state::AppState,
    disk::models::{DiskItemDetail, DiskScanResult, DiskVolumeInfo, ScanPreview},
};

#[tauri::command]
pub fn get_disk_volume_info(state: State<AppState>) -> Result<DiskVolumeInfo, String> {
    Ok(state.disk.volume_info())
}

#[tauri::command]
pub fn get_disk_scan_preview(state: State<AppState>) -> Result<ScanPreview, String> {
    Ok(state.disk.scan_preview())
}

#[tauri::command]
pub fn get_disk_scan_result(state: State<AppState>) -> Result<Option<DiskScanResult>, String> {
    Ok(state.disk.get_scan_result())
}

#[tauri::command]
pub fn get_disk_item_detail(
    state: State<AppState>,
    item_id: String,
) -> Result<DiskItemDetail, String> {
    state.disk.get_item_detail(&item_id)
}

#[tauri::command]
pub async fn start_disk_scan(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<DiskScanResult, String> {
    state.disk.run_scan(app).await
}

#[tauri::command]
pub async fn delete_disk_items(
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<u64, String> {
    state.disk.delete_items(ids).await
}

#[tauri::command]
pub async fn delete_all_disk_items(state: State<'_, AppState>) -> Result<u64, String> {
    let result = state
        .disk
        .get_scan_result()
        .ok_or_else(|| "No scan results available".to_string())?;

    let ids: Vec<String> = result
        .categories
        .iter()
        .flat_map(|group| group.items.iter())
        .filter(|item| {
            item.safety != crate::disk::models::DiskSafety::Unsafe
                && item.action == crate::disk::models::CleanupAction::Delete
        })
        .map(|item| item.id.clone())
        .collect();

    state.disk.delete_items(ids).await
}
