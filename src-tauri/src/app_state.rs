use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    clipboard::{ClipboardService, ClipboardStorage},
    disk::{DiskService, DiskStorage},
    license::{client::LicenseClient, LicenseService, LicenseStorage},
    migration::run_migrations,
    preferences::{service::PreferencesService, PreferencesStorage},
    workspaces::{WorkspaceService, WorkspaceStorage},
};

#[derive(Clone)]
pub struct AppState {
    pub preferences: Arc<PreferencesService>,
    pub clipboard: Arc<ClipboardService>,
    pub workspaces: Arc<WorkspaceService>,
    pub disk: Arc<DiskService>,
    pub license: Arc<LicenseService>,
    pub app_handle: AppHandle,

    pub clipboard_images_dir: PathBuf,
}

impl AppState {
    pub fn new(
        preferences: PreferencesService,
        clipboard: Arc<ClipboardService>,
        workspaces: Arc<WorkspaceService>,
        disk: Arc<DiskService>,
        license: Arc<LicenseService>,
        app_handle: AppHandle,
        clipboard_images_dir: PathBuf,
    ) -> Self {
        Self {
            preferences: Arc::new(preferences),
            clipboard,
            workspaces,
            disk,
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

    let license_client = LicenseClient::new("http://localhost:8000/");

    let license = Arc::new(LicenseService::new(
        LicenseStorage::new(db_path.clone())?,
        license_client,
    ));
    let clipboard = Arc::new(ClipboardService::new(ClipboardStorage::new(
        db_path.clone(),
    )?));
    let workspaces = Arc::new(WorkspaceService::new(WorkspaceStorage::new(
        db_path.clone(),
    )?));
    let disk = Arc::new(DiskService::new(DiskStorage::new(app_dir.clone())));
    let preferences_service = PreferencesService::new(storage, license.clone(), clipboard.clone());

    Ok(AppState::new(
        preferences_service,
        clipboard,
        workspaces,
        disk,
        license,
        app.handle().clone(),
        images_dir,
    ))
}
