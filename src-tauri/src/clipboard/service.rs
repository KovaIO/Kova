use rusqlite::Result;
use std::sync::Mutex;

use crate::clipboard::{
    models::{ClipboardContentType, ClipboardItem},
    storage::{effective_history_limit, ClipboardStorage},
    watcher::cleanup_image_files,
};

pub struct ClipboardService {
    storage: Mutex<ClipboardStorage>,
}

impl ClipboardService {
    pub fn new(storage: ClipboardStorage) -> Self {
        Self {
            storage: Mutex::new(storage),
        }
    }

    pub fn list_history(
        &self,
        limit: i32,
        search: Option<&str>,
    ) -> Result<Vec<ClipboardItem>, String> {
        let removed = self.trim_to_limit(limit)?;
        cleanup_image_files(&removed);

        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        let max = effective_history_limit(limit);
        storage.list_history(max, search).map_err(|e| e.to_string())
    }

    pub fn get_item(&self, id: i64) -> Result<Option<ClipboardItem>, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.get_item(id).map_err(|e| e.to_string())
    }

    pub fn delete_item(&self, id: i64) -> Result<Option<String>, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.delete_item(id).map_err(|e| e.to_string())
    }

    pub fn clear_history(&self) -> Result<Vec<String>, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.clear_history().map_err(|e| e.to_string())
    }

    pub fn insert_item(
        &self,
        content_type: ClipboardContentType,
        text_content: Option<&str>,
        image_path: Option<&str>,
        source_app: Option<&str>,
        source_app_path: Option<&str>,
        created_at: i64,
    ) -> Result<i64, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage
            .insert_item(
                content_type,
                text_content,
                image_path,
                source_app,
                source_app_path,
                created_at,
            )
            .map_err(|e| e.to_string())
    }

    pub fn latest_matches(
        &self,
        content_type: ClipboardContentType,
        text_content: Option<&str>,
        image_path: Option<&str>,
    ) -> Result<bool, String> {
        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage
            .latest_matches(content_type, text_content, image_path)
            .map_err(|e| e.to_string())
    }

    pub fn trim_to_limit(&self, limit: i32) -> Result<Vec<String>, String> {
        let max = effective_history_limit(limit);
        if max <= 0 || max >= i32::MAX / 4 {
            return Ok(vec![]);
        }

        let storage = self.storage.lock().map_err(|e| e.to_string())?;
        storage.trim_to_limit(max).map_err(|e| e.to_string())
    }
}
