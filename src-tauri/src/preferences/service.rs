use rusqlite::Result;
use std::sync::{Arc, Mutex};

use crate::{
    license::{
        sanitize_clipboard_history_limit, sanitize_window_manager_preferences,
        validate_clipboard_history_limit, validate_monitor_dim,
        validate_window_manager_preferences, LicenseService,
    },
    preferences::{
        ClipboardPreferences, GeneralPreferences, Preferences, PreferencesStorage,
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
            .map_err(|e| e.to_string())
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
}
