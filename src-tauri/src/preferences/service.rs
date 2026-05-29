use rusqlite::Result;
use std::sync::Mutex;

use crate::preferences::{
    ClipboardPreferences, GeneralPreferences, Preferences, PreferencesStorage,
    WindowManagerPreferences,
};

pub struct PreferencesService {
    storage: Mutex<PreferencesStorage>,
}

impl PreferencesService {
    pub fn new(storage: PreferencesStorage) -> Self {
        Self {
            storage: Mutex::new(storage),
        }
    }

    pub fn get_preferences(&self) -> Result<Preferences> {
        let storage = self.storage.lock().unwrap();
        storage.load_preferences()
    }

    pub fn update_general_preferences(&self, prefs: GeneralPreferences) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_general_preferences(&prefs)
    }

    pub fn update_clipboard_preferences(&self, prefs: ClipboardPreferences) -> Result<()> {
        let mut validated = prefs;
        validated.history_limit = validated.history_limit.min(25);
        let storage = self.storage.lock().unwrap();
        storage.save_clipboard_preferences(&validated)
    }

    pub fn update_window_manager_preferences(&self, prefs: WindowManagerPreferences) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_window_manager_preferences(&prefs)
    }
}
