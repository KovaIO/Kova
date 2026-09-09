use rusqlite::Result;
use std::sync::{Arc, Mutex};

use crate::{
    clipboard::{service::ClipboardService, watcher::cleanup_image_files},
    preferences::{
        monitor, AppearancePreferences, ClipboardPreferences, GeneralPreferences, Preferences,
        PreferencesStorage, Shortcut,
    },
};

pub struct PreferencesService {
    storage: Mutex<PreferencesStorage>,
    clipboard: Arc<ClipboardService>,
}

impl PreferencesService {
    pub fn new(storage: PreferencesStorage, clipboard: Arc<ClipboardService>) -> Self {
        Self {
            storage: Mutex::new(storage),
            clipboard,
        }
    }

    pub fn get_preferences(&self) -> Result<Preferences> {
        let storage = self.storage.lock().unwrap();
        storage.load_preferences()
    }

    pub fn get_shortcuts(&self) -> Result<Vec<Shortcut>> {
        let storage = self.storage.lock().unwrap();

        storage.load_shortcuts()
    }

    pub fn update_general_preferences(&self, prefs: GeneralPreferences) -> Result<(), String> {
        let storage = self.storage.lock().unwrap();

        storage
            .save_general_preferences(&prefs)
            .map_err(|e| e.to_string())?;

        if let Err(err) = monitor::set_brightness(prefs.monitor_dim) {
            eprintln!("Failed to set brightness: {}", err);
        }

        Ok(())
    }

    pub fn update_clipboard_preferences(&self, prefs: ClipboardPreferences) -> Result<(), String> {
        let storage = self.storage.lock().unwrap();
        storage
            .save_clipboard_preferences(&prefs)
            .map_err(|e| e.to_string())?;

        if prefs.history_limit > 0 {
            let removed = self.clipboard.trim_to_limit(prefs.history_limit)?;
            cleanup_image_files(&removed);
        }

        Ok(())
    }

    pub fn update_shortcuts(&self, shortcuts: Vec<Shortcut>) -> Result<(), String> {
        let storage = self.storage.lock().unwrap();

        storage
            .save_shortcuts(&shortcuts)
            .map_err(|e| e.to_string())
    }

    pub fn update_appearance_preferences(
        &self,
        prefs: AppearancePreferences,
    ) -> Result<(), String> {
        let storage = self.storage.lock().unwrap();

        storage
            .save_appearance_preferences(&prefs)
            .map_err(|e| e.to_string())
    }

    pub fn save_monitor_dim(&self, dim: u8) -> Result<()> {
        let storage = self.storage.lock().unwrap();
        storage.save_monitor_dim(dim)
    }
}
