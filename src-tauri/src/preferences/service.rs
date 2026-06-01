use rusqlite::Result;
use std::sync::{Arc, Mutex};

use crate::{
    clipboard::{
        models::ClipboardItem,
        storage::{
            clear_history, delete_item, effective_history_limit, get_item, list_history,
            trim_to_limit,
        },
        watcher::cleanup_image_files,
    },
    license::{
        sanitize_clipboard_history_limit, sanitize_window_manager_preferences,
        validate_clipboard_history_limit, validate_monitor_dim,
        validate_window_manager_preferences, LicenseService,
    },
    preferences::{
        ClipboardPreferences, GeneralPreferences, Preferences, PreferencesStorage, Shortcut,
        WindowManagerPreferences,
    },
};

pub struct PreferencesService {
    storage: Mutex<PreferencesStorage>,
    license: Arc<LicenseService>,
}

impl PreferencesService {
    pub fn new(storage: PreferencesStorage, license: Arc<LicenseService>) -> Self {
        Self {
            storage: Mutex::new(storage),
            license,
        }
    }

    pub fn get_preferences(&self) -> Result<Preferences> {
        let storage = self.storage.lock().unwrap();
        let mut preferences = storage.load_preferences()?;
        let tier = self.license.tier()?;
        preferences.clipboard.history_limit =
            sanitize_clipboard_history_limit(preferences.clipboard.history_limit, &tier);
        preferences.window_manager =
            sanitize_window_manager_preferences(preferences.window_manager, &tier);
        Ok(preferences)
    }

    pub fn get_shortcuts(&self) -> Result<Vec<Shortcut>> {
        let storage = self.storage.lock().unwrap();

        storage.load_shortcuts()
    }

    pub fn update_general_preferences(&self, prefs: GeneralPreferences) -> Result<(), String> {
        let tier = self.license.tier().map_err(|e| e.to_string())?;
        let storage = self.storage.lock().unwrap();
        let current = storage
            .load_general_preferences()
            .map_err(|e| e.to_string())?;

        let monitor_dim = if prefs.monitor_dim != current.monitor_dim {
            validate_monitor_dim(prefs.monitor_dim, &tier)?
        } else {
            current.monitor_dim
        };

        let validated = GeneralPreferences {
            monitor_dim,
            ..prefs
        };

        storage
            .save_general_preferences(&validated)
            .map_err(|e| e.to_string())
    }

    pub fn update_clipboard_preferences(&self, prefs: ClipboardPreferences) -> Result<(), String> {
        let tier = self.license.tier().map_err(|e| e.to_string())?;
        let history_limit = validate_clipboard_history_limit(prefs.history_limit, &tier)?;

        let validated = ClipboardPreferences {
            history_limit,
            ..prefs
        };

        let storage = self.storage.lock().unwrap();
        storage
            .save_clipboard_preferences(&validated)
            .map_err(|e| e.to_string())?;

        if validated.history_limit > 0 {
            let max = effective_history_limit(validated.history_limit);
            if max < i32::MAX / 4 {
                let removed =
                    trim_to_limit(storage.connection(), max).map_err(|e| e.to_string())?;
                cleanup_image_files(&removed);
            }
        }

        Ok(())
    }

    pub fn update_window_manager_preferences(
        &self,
        prefs: WindowManagerPreferences,
    ) -> Result<(), String> {
        let tier = self.license.tier().map_err(|e| e.to_string())?;
        let validated = validate_window_manager_preferences(prefs, &tier)?;

        let storage = self.storage.lock().unwrap();
        storage
            .save_window_manager_preferences(&validated)
            .map_err(|e| e.to_string())
    }

    pub fn update_shortcuts(&self, shortcuts: Vec<Shortcut>) -> Result<(), String> {
        let storage = self.storage.lock().unwrap();

        storage
            .save_shortcuts(&shortcuts)
            .map_err(|e| e.to_string())
    }

    pub fn with_storage<R>(
        &self,
        f: impl FnOnce(&PreferencesStorage) -> Result<R, String>,
    ) -> Result<R, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        f(&storage)
    }

    pub fn list_clipboard_history(
        &self,
        limit: i32,
        search: Option<&str>,
    ) -> Result<Vec<ClipboardItem>, String> {
        self.with_storage(|storage| {
            let conn = storage.connection();
            let max = effective_history_limit(limit);
            if max > 0 && max < i32::MAX / 4 {
                let removed = trim_to_limit(conn, max).map_err(|e| e.to_string())?;
                cleanup_image_files(&removed);
            }
            list_history(conn, max, search).map_err(|e| e.to_string())
        })
    }

    pub fn get_clipboard_item(&self, id: i64) -> Result<Option<ClipboardItem>, String> {
        self.with_storage(|storage| get_item(storage.connection(), id).map_err(|e| e.to_string()))
    }

    pub fn delete_clipboard_item(&self, id: i64) -> Result<Option<String>, String> {
        self.with_storage(|storage| {
            delete_item(storage.connection(), id).map_err(|e| e.to_string())
        })
    }

    pub fn clear_clipboard_history(&self) -> Result<Vec<String>, String> {
        self.with_storage(|storage| clear_history(storage.connection()).map_err(|e| e.to_string()))
    }
}
