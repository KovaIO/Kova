use arboard::{Clipboard, ImageData};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    app_state::AppState,
    clipboard::{
        get_installed_apps,
        models::{ClipboardContentType, ClipboardItem, InstalledApp},
        paste::simulate_paste,
        watcher::{
            clear_history_files, delete_item_files, remember_image_signature, remember_text,
        },
    },
};

#[cfg(target_os = "windows")]
use crate::clipboard::paste::set_image_to_clipboard_delayed;

#[tauri::command]
pub fn get_apps() -> Vec<InstalledApp> {
    get_installed_apps()
}

#[tauri::command]
pub fn get_clipboard_history(
    state: State<AppState>,
    search: Option<String>,
) -> Result<Vec<ClipboardItem>, String> {
    let prefs = state
        .preferences
        .get_preferences()
        .map_err(|e| e.to_string())?;
    state
        .clipboard
        .list_history(prefs.clipboard.history_limit, search.as_deref())
}

#[tauri::command]
pub async fn paste_clipboard_item(
    state: State<'_, AppState>,
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    let item = get_required_item(&state, id)?;

    if let Some(window) = app.get_webview_window("clipboard") {
        let _ = window.hide();
    }

    tokio::task::spawn_blocking(move || {
        let success = match item.content_type {
            ClipboardContentType::Text => {
                if let Some(text) = item.text_content {
                    let mut clipboard = match Clipboard::new() {
                        Ok(c) => c,
                        Err(e) => { eprintln!("Clipboard error: {e}"); return; }
                    };
                    let ok = clipboard.set_text(text.clone()).is_ok();
                    if ok { remember_text(text); }
                    ok
                } else { false }
            }
            ClipboardContentType::Image => {
                if let Some(path) = item.image_path {
                    #[cfg(target_os = "windows")]
                    {
                        match set_image_to_clipboard_delayed(path) {
                            Ok(sig) => { remember_image_signature(sig); true }
                            Err(e) => { eprintln!("Image clipboard error: {e}"); false }
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let mut clipboard = match Clipboard::new() {
                            Ok(c) => c,
                            Err(e) => { eprintln!("Clipboard error: {e}"); return; }
                        };
                        match set_clipboard_image(&mut clipboard, &path) {
                            Ok(sig) => { remember_image_signature(sig); true }
                            Err(e) => { eprintln!("Image clipboard error: {e}"); false }
                        }
                    }
                } else { false }
            }
        };

        if success {
            std::thread::sleep(std::time::Duration::from_millis(150));
            simulate_paste();
        }
    });

    Ok(())
}

#[tauri::command]
pub fn copy_clipboard_item(state: State<AppState>, id: i64) -> Result<(), String> {
    write_item_to_clipboard(&state, id)
}

#[tauri::command]
pub async fn paste_plain_clipboard_item(
    state: State<'_, AppState>,
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    let item = get_required_item(&state, id)?;
    let text = plain_text_for_item(&item)?;

    if let Some(window) = app.get_webview_window("clipboard") {
        let _ = window.hide();
    }

    tokio::task::spawn_blocking(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => { eprintln!("Clipboard error: {e}"); return; }
        };
        if clipboard.set_text(text.clone()).is_ok() {
            remember_text(text);
            std::thread::sleep(std::time::Duration::from_millis(150));
            simulate_paste();
        }
    });

    Ok(())
}

#[tauri::command]
pub fn open_clipboard_url(state: State<AppState>, app: AppHandle, id: i64) -> Result<(), String> {
    let item = get_required_item(&state, id)?;
    let url = extract_url(&item).ok_or_else(|| "No URL found in this item".to_string())?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_clipboard_item(
    state: State<AppState>,
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    let item = get_required_item(&state, id)?;
    let path = item
        .image_path
        .as_deref()
        .ok_or_else(|| "This item has no file to reveal".to_string())?;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_clipboard_item(
    state: State<AppState>,
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    let item = get_required_item(&state, id)?;
    let path = item
        .image_path
        .as_deref()
        .ok_or_else(|| "This item has no file to preview".to_string())?;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_clipboard_item(
    state: State<AppState>,
    app: AppHandle,
    id: i64,
) -> Result<(), String> {
    delete_item_files(&state, id)?;
    let _ = app.emit("clipboard-history-updated", ());
    Ok(())
}

#[tauri::command]
pub fn clear_clipboard_history(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    clear_history_files(&state)?;
    let _ = app.emit("clipboard-history-updated", ());
    Ok(())
}

fn get_required_item(state: &AppState, id: i64) -> Result<ClipboardItem, String> {
    state
        .clipboard
        .get_item(id)?
        .ok_or_else(|| "Clipboard item not found".to_string())
}

fn write_item_to_clipboard(state: &AppState, id: i64) -> Result<(), String> {
    let item = get_required_item(state, id)?;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match item.content_type {
        ClipboardContentType::Text => {
            let text = item
                .text_content
                .ok_or_else(|| "Clipboard item has no text".to_string())?;
            clipboard.set_text(text.clone()).map_err(|e| e.to_string())?;
            remember_text(text);
        }
        ClipboardContentType::Image => {
            let path = item
                .image_path
                .ok_or_else(|| "Clipboard item has no image".to_string())?;
            let signature = set_clipboard_image(&mut clipboard, &path)?;
            remember_image_signature(signature);
        }
    }

    Ok(())
}

fn plain_text_for_item(item: &ClipboardItem) -> Result<String, String> {
    match item.content_type {
        ClipboardContentType::Text => item
            .text_content
            .clone()
            .ok_or_else(|| "Clipboard item has no text".to_string()),
        ClipboardContentType::Image => Ok(item.image_path.clone().unwrap_or_default()),
    }
}

fn extract_url(item: &ClipboardItem) -> Option<String> {
    let text = item.text_content.as_deref()?.trim();
    if text.starts_with("http://") || text.starts_with("https://") {
        Some(text.to_string())
    } else {
        None
    }
}

fn set_clipboard_image(clipboard: &mut Clipboard, path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw = rgba.into_raw();
    let signature = format!("{}x{}:{}", width, height, raw.len());
    let data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: raw.into(),
    };
    clipboard.set_image(data).map_err(|e| e.to_string())?;
    Ok(signature)
}