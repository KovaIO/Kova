use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use arboard::{Clipboard, ImageData};
use image::ImageEncoder;
use tauri::{AppHandle, Emitter};

use crate::{
    app_state::AppState,
    clipboard::{
        models::{ClipboardContentType, SourceApp},
        source::{get_foreground_app, is_app_ignored},
        storage,
    },
};

static WATCHER_STARTED: AtomicBool = AtomicBool::new(false);

static LAST_CAPTURE: Mutex<LastCapture> = Mutex::new(LastCapture {
    text: None,
    image_signature: None,
});

#[derive(Default)]
struct LastCapture {
    text: Option<String>,
    image_signature: Option<String>,
}

pub fn remember_text(text: String) {
    if let Ok(mut guard) = LAST_CAPTURE.lock() {
        guard.text = Some(text);
        guard.image_signature = None;
    }
}

pub fn remember_image_signature(signature: String) {
    if let Ok(mut guard) = LAST_CAPTURE.lock() {
        guard.image_signature = Some(signature);
        guard.text = None;
    }
}

pub fn start(app: AppHandle, state: AppState) {
    if WATCHER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(err) => {
                eprintln!("clipboard watcher: failed to open clipboard: {err}");
                return;
            }
        };

        loop {
            std::thread::sleep(Duration::from_millis(450));

            let prefs = match state.preferences.get_preferences() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if !prefs.clipboard.enabled {
                continue;
            }

            let source = get_foreground_app();

            if let Some(source) = &source {
                if is_own_process(source) || is_app_ignored(source, &prefs.clipboard.ignored_apps) {
                    continue;
                }
            }

            if let Err(err) = poll_once(&app, &state, &mut clipboard, &source) {
                eprintln!("clipboard watcher: {err}");
            }
        }
    });
}

fn is_own_process(source: &SourceApp) -> bool {
    if let Ok(exe) = std::env::current_exe() {
        let own = exe.to_string_lossy().to_lowercase();
        let current = source.path.to_lowercase();
        if !current.is_empty() && current == own {
            return true;
        }
    }

    source.name.eq_ignore_ascii_case("kova") || source.name.eq_ignore_ascii_case("app")
}

fn poll_once(
    app: &AppHandle,
    state: &AppState,
    clipboard: &mut Clipboard,
    source: &Option<SourceApp>,
) -> Result<(), String> {
    if let Ok(image) = clipboard.get_image() {
        if try_capture_image(app, state, &image, source)? {
            return Ok(());
        }
    }

    if let Ok(text) = clipboard.get_text() {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if try_capture_text(app, state, trimmed.to_string(), source)? {
            return Ok(());
        }
    }

    Ok(())
}

fn try_capture_text(
    app: &AppHandle,
    state: &AppState,
    text: String,
    source: &Option<SourceApp>,
) -> Result<bool, String> {
    {
        let guard = LAST_CAPTURE.lock().map_err(|e| e.to_string())?;
        if guard.text.as_ref() == Some(&text) {
            return Ok(false);
        }
    }

    let prefs = state
        .preferences
        .get_preferences()
        .map_err(|e| e.to_string())?;

    if prefs.clipboard.ignore_passwords && looks_like_password(&text) {
        return Ok(false);
    }

    let is_duplicate = state
        .preferences
        .with_storage(|storage| {
            storage::latest_matches(
                storage.connection(),
                ClipboardContentType::Text,
                Some(&text),
                None,
            )
            .map_err(|e| e.to_string())
        })?;

    if is_duplicate {
        return Ok(false);
    }

    let created_at = chrono_timestamp();
    let source_name = source.as_ref().map(|s| s.name.as_str());
    let source_path = source
        .as_ref()
        .map(|s| s.path.as_str())
        .filter(|p| !p.is_empty());

    state.preferences.with_storage(|storage| {
        let conn = storage.connection();
        storage::insert_item(
            conn,
            ClipboardContentType::Text,
            Some(&text),
            None,
            source_name,
            source_path,
            created_at,
        )
        .map_err(|e| e.to_string())?;
        storage::trim_to_limit(conn, effective_limit(prefs.clipboard.history_limit))
            .map_err(|e| e.to_string())
    })?;

    remember_text(text);

    emit_updated(app);
    Ok(true)
}

fn try_capture_image(
    app: &AppHandle,
    state: &AppState,
    image: &ImageData,
    source: &Option<SourceApp>,
) -> Result<bool, String> {
    let signature = format!("{}x{}:{}", image.width, image.height, image.bytes.len());

    {
        let guard = LAST_CAPTURE.lock().map_err(|e| e.to_string())?;
        if guard.image_signature.as_ref() == Some(&signature) {
            return Ok(false);
        }
    }

    let path = save_image(state, image)?;

    let prefs = state
        .preferences
        .get_preferences()
        .map_err(|e| e.to_string())?;

    let is_duplicate = state
        .preferences
        .with_storage(|storage| {
            storage::latest_matches(
                storage.connection(),
                ClipboardContentType::Image,
                None,
                Some(&path),
            )
            .map_err(|e| e.to_string())
        })?;

    if is_duplicate {
        let _ = std::fs::remove_file(&path);
        return Ok(false);
    }

    let created_at = chrono_timestamp();
    let source_name = source.as_ref().map(|s| s.name.as_str());
    let source_path = source
        .as_ref()
        .map(|s| s.path.as_str())
        .filter(|p| !p.is_empty());

    let removed = state.preferences.with_storage(|storage| {
        let conn = storage.connection();
        storage::insert_item(
            conn,
            ClipboardContentType::Image,
            None,
            Some(&path),
            source_name,
            source_path,
            created_at,
        )
        .map_err(|e| e.to_string())?;
        storage::trim_to_limit(conn, effective_limit(prefs.clipboard.history_limit))
            .map_err(|e| e.to_string())
    })?;

    cleanup_image_files(&removed);

    remember_image_signature(signature);

    emit_updated(app);
    Ok(true)
}

fn save_image(state: &AppState, image: &ImageData) -> Result<String, String> {
    let file_name = format!("{}.png", uuid_simple());
    let path = state.clipboard_images_dir.join(&file_name);

    let img = image::RgbaImage::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
        .ok_or_else(|| "invalid clipboard image dimensions".to_string())?;

    let mut buffer = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;

    std::fs::write(&path, buffer).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

fn effective_limit(limit: i32) -> i32 {
    if limit <= 0 {
        i32::MAX / 2
    } else {
        limit
    }
}

fn looks_like_password(text: &str) -> bool {
    if text.len() > 128 {
        return false;
    }

    let has_upper = text.chars().any(|c| c.is_uppercase());
    let has_lower = text.chars().any(|c| c.is_lowercase());
    let has_digit = text.chars().any(|c| c.is_ascii_digit());
    let has_symbol = text.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace());

    text.len() >= 12 && has_upper && has_lower && has_digit && has_symbol
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    format!("{:x}", nanos)
}

fn emit_updated(app: &AppHandle) {
    let _ = app.emit("clipboard-history-updated", ());
}

pub fn cleanup_image_files(paths: &[String]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

pub fn clear_history_files(state: &AppState) -> Result<(), String> {
    let paths = state
        .preferences
        .clear_clipboard_history()
        .map_err(|e| e.to_string())?;
    cleanup_image_files(&paths);
    Ok(())
}

pub fn delete_item_files(state: &AppState, id: i64) -> Result<(), String> {
    if let Some(path) = state.preferences.delete_clipboard_item(id)? {
        cleanup_image_files(&[path]);
    }
    Ok(())
}
