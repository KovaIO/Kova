use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Clone, Serialize)]
pub struct PendingUpdate {
    pub version: String,
    pub notes: String,
}

pub struct UpdateState {
    pub pending: Mutex<Option<PendingUpdate>>,
    pub downloading: Mutex<bool>,
}

impl UpdateState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(None),
            downloading: Mutex::new(false),
        }
    }
}

pub async fn check_for_updates(app: AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return,
    };

    let update = match updater.check().await {
        Ok(Some(u)) => u,
        _ => return,
    };

    let notes = update.body.clone().unwrap_or_default();

    let state = UpdateState {
        pending: Mutex::new(Some(PendingUpdate {
            version: update.version.clone(),
            notes,
        })),
        downloading: Mutex::new(false),
    };

    app.manage(state);

    if let Some(win) = app.get_webview_window("update") {
        if let Ok(Some(monitor)) = win.current_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size();
            let margin = 16.0_f64;
            let win_width = 400.0_f64;

            let x = ((size.width as f64 / scale) - win_width - margin) * scale;
            let y = margin * scale;

            let _ = win.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
        }
        let _ = win.show();
    }
}

#[tauri::command]
pub fn get_update(app: AppHandle) -> Option<PendingUpdate> {
    let state = app.state::<UpdateState>();
    let guard = state.pending.lock().ok()?;
    guard.clone()
}

#[tauri::command]
pub fn dismiss_update(app: AppHandle) {
    if let Some(w) = app.get_webview_window("update") {
        crate::windows::hide_window(&w);
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let state = app.state::<UpdateState>();
    {
        let mut dl = state.downloading.lock().map_err(|e| e.to_string())?;
        if *dl {
            return Ok(());
        }
        *dl = true;
    }

    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    let update = update.ok_or("no update available")?;

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}
