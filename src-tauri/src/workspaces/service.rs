use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Result;

use crate::workspaces::{
    applier::{apply_window, focus_window, minimize_other_windows},
    launch_app,
    matcher::{find_all_windows, MatchedWindow},
    WorkspaceProfile, WorkspaceStorage,
};

pub struct WorkspaceService {
    storage: Mutex<WorkspaceStorage>,
}

fn is_browser_app(path: &str) -> bool {
    path.contains("chrome")
        || path.contains("firefox")
        || path.contains("msedge")
        || path.contains("brave")
        || path.contains("opera")
        || path.contains("vivaldi")
        || path.contains("arc")
        || path.contains("waterfox")
        || path.contains("librewolf")
        || path.contains("zen")
}

fn extract_domain_keyword(url: &str) -> Option<String> {
    let without_proto = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let domain = without_proto.split('/').next()?;
    let keyword = domain
        .split('.')
        .next()
        .unwrap_or(domain)
        .to_lowercase();
    if keyword.is_empty() { None } else { Some(keyword) }
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

        // Separate apps into browser and non-browser
        struct BrowserEntry {
            layout_idx: usize,
        }
        struct NonBrowserGroup {
            indices: Vec<usize>,
            launch_target: String,
        }

        let mut browser_entries: Vec<BrowserEntry> = Vec::new();
        let mut non_browser_groups: HashMap<String, NonBrowserGroup> = HashMap::new();

        for (i, app) in profile.apps.iter().enumerate() {
            let key = app.exe_path.as_deref().unwrap_or(&app.path).to_lowercase();
            if is_browser_app(&key) {
                browser_entries.push(BrowserEntry { layout_idx: i });
            } else {
                let entry = non_browser_groups
                    .entry(key)
                    .or_insert_with(|| NonBrowserGroup {
                        indices: Vec::new(),
                        launch_target: app.exe_path.as_deref().unwrap_or(&app.path).to_string(),
                    });
                entry.indices.push(i);
            }
        }

        // Phase 1: Minimize all other windows
        minimize_other_windows(&std::collections::HashSet::new());

        let mut final_windows: Vec<Option<MatchedWindow>> =
            (0..profile.apps.len()).map(|_| None).collect();

        // Phase 2a: Non-browser groups — match existing windows, launch missing
        let mut global_used_windows: Vec<usize> = Vec::new();

        for (_key, group) in &non_browser_groups {
            let reference_app = &profile.apps[group.indices[0]];
            let open_windows = find_all_windows(reference_app);

            let mut unmatched_indices: Vec<usize> = Vec::new();
            for &idx in &group.indices {
                if let Some(window) = open_windows.iter().enumerate().find_map(|(j, w)| {
                    if global_used_windows.contains(&j) {
                        None
                    } else {
                        global_used_windows.push(j);
                        Some(w.clone())
                    }
                }) {
                    final_windows[idx] = Some(window);
                } else {
                    unmatched_indices.push(idx);
                }
            }

            for &idx in &unmatched_indices {
                let app = &profile.apps[idx];
                let _ = launch_app(&group.launch_target, &app.urls);
            }
        }

        // Phase 2b: Browser entries — each gets its own window
        // Step 1: Collect all unique browser windows (once, not per-entry)
        let mut all_browser_windows: Vec<MatchedWindow> = Vec::new();
        for entry in &browser_entries {
            let ref_app = &profile.apps[entry.layout_idx];
            for w in find_all_windows(ref_app) {
                if !all_browser_windows.iter().any(|bw| bw.handle == w.handle) {
                    all_browser_windows.push(w);
                }
            }
        }

        // Step 2: Match windows to entries by title containing URL domain keyword
        let mut matched_windows: Vec<Option<MatchedWindow>> =
            (0..browser_entries.len()).map(|_| None).collect();
        let mut unmatched_indices: Vec<usize> = Vec::new();

        for (i, entry) in browser_entries.iter().enumerate() {
            let urls = &profile.apps[entry.layout_idx].urls;
            let keywords: Vec<String> = urls
                .iter()
                .filter_map(|url| extract_domain_keyword(url))
                .collect();

            if let Some(pos) = all_browser_windows.iter().position(|w| {
                let title_lower = w.title.to_lowercase();
                keywords.iter().any(|k| title_lower.contains(k))
            }) {
                let w = all_browser_windows.remove(pos);
                matched_windows[i] = Some(w);
            } else {
                unmatched_indices.push(i);
            }
        }

        // Step 3: Assign matched windows
        for (i, entry) in browser_entries.iter().enumerate() {
            if let Some(w) = matched_windows[i].take() {
                final_windows[entry.layout_idx] = Some(w);
            }
        }

        // Step 4: Launch unmatched entries one at a time with delay
        for &idx in &unmatched_indices {
            let app = &profile.apps[browser_entries[idx].layout_idx];
            let _ = launch_app(
                &app.exe_path.as_deref().unwrap_or(&app.path),
                &app.urls,
            );
            if unmatched_indices.len() > 1 {
                std::thread::sleep(std::time::Duration::from_millis(500));
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
                    apply_window(&app_handle, &app, &window, profile_gap);
                } else {
                    // Newly launched — poll every 100ms until found (up to 3s)
                    for _ in 0..30 {
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
