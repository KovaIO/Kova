use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
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

    let state = app.state::<UpdateState>();
    {
        let mut pending = state.pending.lock().unwrap();
        *pending = Some(PendingUpdate {
            version: update.version.clone(),
            notes,
        });
    }

    if app.get_webview_window("update").is_none() {
        if let Ok(w) = WebviewWindowBuilder::new(&app, "update", WebviewUrl::App("/update".into()))
            .title("Update Available")
            .inner_size(400.0, 300.0)
            .decorations(false)
            .shadow(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .devtools(false)
            .center()
            .build()
        {
            crate::windows::attach_focus_hide(w);
        }
    }

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
