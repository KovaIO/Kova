use std::sync::Mutex;

use rusqlite::Result;

use crate::workspaces::{
    applier::{apply_window, focus_window},
    launch_app,
    matcher::find_window,
    WorkspaceProfile, WorkspaceStorage,
};

pub struct WorkspaceService {
    storage: Mutex<WorkspaceStorage>,
}

impl WorkspaceService {
    pub fn new(storage: WorkspaceStorage) -> Self {
        Self {
            storage: Mutex::new(storage),
        }
    }

    pub fn get_profiles(&self) -> Result<Vec<WorkspaceProfile>> {
        let storage = self.storage.lock().unwrap();
        storage.load_profiles()
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<WorkspaceProfile>> {
        let storage = self.storage.lock().unwrap();
        storage.load_profile(profile_id)
    }

    pub fn save_profile(&self, profile: WorkspaceProfile) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_profile(&profile)
    }

    pub fn delete_profile(&self, profile_id: &str) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.delete_profile(profile_id)
    }

    pub async fn apply_profile(
        &self,
        profile_id: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        let profile = self
            .get_profile(profile_id)
            .map_err(|e| e.to_string())?
            .ok_or("Profile not found")?;

        let mut any_launched = false;
        for app in &profile.apps {
            if find_window(app).is_some() {
                continue;
            }

            let launch_target = app.exe_path.as_deref().unwrap_or(&app.path);
            let _ = launch_app(launch_target);
            any_launched = true;
        }

        let mut handles = Vec::new();
        let profile_apps = profile.apps.clone();

        for app in profile.apps.clone() {
            let app_handle = app_handle.clone();
            let handle = tauri::async_runtime::spawn(async move {
                let mut matched = None;
                for _attempt in 0..20 {
                    if let Some(window) = find_window(&app) {
                        matched = Some(window);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }

                match matched {
                    Some(window) => {
                        apply_window(&app_handle, &app, &window);
                    }
                    None => {}
                }
            });
            handles.push(handle);
        }

        for h in handles {
            let _ = h.await;
        }

        if any_launched {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        for app in &profile_apps {
            if let Some(window) = find_window(app) {
                focus_window(window.handle, &app.name);
            }
        }

        Ok(())
    }
}
