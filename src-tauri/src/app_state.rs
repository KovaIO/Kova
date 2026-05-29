use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter, Manager};

use crate::preferences::{
    migration::run_migrations, service::PreferencesService, PreferencesStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub preferences: Arc<PreferencesService>,
    pub app_handle: AppHandle,
}

impl AppState {
    pub fn new(preferences: PreferencesService, app_handle: AppHandle) -> Self {
        Self {
            preferences: Arc::new(preferences),
            app_handle,
        }
    }

    pub fn emit_preferences_updated(&self) {
        if let Ok(prefs) = self.preferences.get_preferences() {
            let _ = self.app_handle.emit("preferences-updated", prefs);
        }
    }
}

pub fn initialize_app_state(app: &tauri::App) -> Result<AppState, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;

    std::fs::create_dir_all(&app_dir)?;

    let db_path: PathBuf = app_dir.join("kova.db");

    let storage = PreferencesStorage::new(db_path)?;
    run_migrations(storage.connection())?;

    let preferences_service = PreferencesService::new(storage);

    Ok(AppState::new(preferences_service, app.handle().clone()))
}
