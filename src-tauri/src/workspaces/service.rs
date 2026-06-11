use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Result;

use crate::workspaces::{
    applier::{apply_window, focus_window, minimize_other_windows},
    launch_app,
    matcher::find_all_windows,
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

        // Group layout entries by app identity
        struct AppGroup {
            indices: Vec<usize>,
            launch_target: String,
        }

        let mut groups: HashMap<String, AppGroup> = HashMap::new();
        for (i, app) in profile.apps.iter().enumerate() {
            let key = app.exe_path.as_deref().unwrap_or(&app.path).to_lowercase();
            let entry = groups.entry(key).or_insert_with(|| AppGroup {
                indices: Vec::new(),
                launch_target: app.exe_path.as_deref().unwrap_or(&app.path).to_string(),
            });
            entry.indices.push(i);
        }

        // Phase 1: Minimize all other windows first (clean desktop)
        minimize_other_windows(&std::collections::HashSet::new());

        // Phase 2: Match existing windows to layout entries, launch what's missing
        let mut final_windows: Vec<Option<crate::workspaces::matcher::MatchedWindow>> =
            (0..profile.apps.len()).map(|_| None).collect();

        for (_key, group) in &groups {
            let reference_app = &profile.apps[group.indices[0]];
            let open_windows = find_all_windows(reference_app);

            let mut used_window_indices: Vec<usize> = Vec::new();
            let mut needed = 0;

            for &idx in &group.indices {
                if let Some(window) = open_windows.iter().enumerate().find_map(|(j, w)| {
                    if used_window_indices.contains(&j) {
                        None
                    } else {
                        used_window_indices.push(j);
                        Some(w.clone())
                    }
                }) {
                    final_windows[idx] = Some(window);
                } else {
                    needed += 1;
                }
            }

            for _ in 0..needed {
                println!("Launching: {}", group.launch_target);
                let _ = launch_app(&group.launch_target);
            }
        }

        // Phase 3: Position existing windows immediately, poll for new ones
        let used_handles = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<usize>::new()));
        let mut handles = Vec::new();
        for i in 0..profile.apps.len() {
            let app = profile.apps[i].clone();
            let app_handle = app_handle.clone();
            let profile_gap = profile.gap;
            let window = final_windows[i].clone();
            let used_handles = used_handles.clone();

            let handle = tauri::async_runtime::spawn(async move {
                if let Some(window) = window {
                    // Already matched — position immediately
                    apply_window(&app_handle, &app, &window, profile_gap);
                } else {
                    // Newly launched — poll every 100ms until found (up to 2s)
                    for _ in 0..20 {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let all = find_all_windows(&app);
                        let mut lock = used_handles.lock().await;
                        if let Some(w) = all.iter().find(|w| !lock.contains(&w.handle)) {
                            lock.push(w.handle);
                            drop(lock);
                            apply_window(&app_handle, &app, w, profile_gap);
                            return;
                        }
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            let _ = h.await;
        }

        // Phase 4: Focus all windows in layout order
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        for app in &profile.apps {
            if let Some(window) = find_all_windows(app).first().cloned() {
                focus_window(window.handle, &app.name);
            }
        }

        Ok(())
    }
}
