use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    license::{LicenseService, LicenseStorage},
    migration::run_migrations,
    preferences::{service::PreferencesService, PreferencesStorage},
};

#[derive(Clone)]
pub struct AppState {
    pub preferences: Arc<PreferencesService>,
    pub license: Arc<LicenseService>,
    pub app_handle: AppHandle,

    pub clipboard_images_dir: PathBuf,
}

impl AppState {
    pub fn new(
        preferences: PreferencesService,
        license: Arc<LicenseService>,
        app_handle: AppHandle,
        clipboard_images_dir: PathBuf,
    ) -> Self {
        Self {
            preferences: Arc::new(preferences),
            license,
            app_handle,
            clipboard_images_dir,
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

    let clipboard_dir = app_dir.join("clipboard");
    let images_dir = clipboard_dir.join("images");

    std::fs::create_dir_all(&images_dir)?;

    let db_path: PathBuf = app_dir.join("kova.db");

    let storage = PreferencesStorage::new(db_path.clone())?;
    run_migrations(storage.connection())?;

    let license = Arc::new(LicenseService::new(LicenseStorage::new(db_path)?));
    let preferences_service = PreferencesService::new(storage, license.clone());

    Ok(AppState::new(
        preferences_service,
        license,
        app.handle().clone(),
        images_dir,
    ))
}
