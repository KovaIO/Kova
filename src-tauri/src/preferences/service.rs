use rusqlite::Result;
use std::sync::{Arc, Mutex};

use crate::{
    license::{sanitize_clipboard_history_limit, validate_clipboard_history_limit, LicenseService},
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
        Ok(preferences)
    }

    pub fn update_general_preferences(&self, prefs: GeneralPreferences) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_general_preferences(&prefs)
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

    pub fn update_window_manager_preferences(&self, prefs: WindowManagerPreferences) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_window_manager_preferences(&prefs)
    }
}
