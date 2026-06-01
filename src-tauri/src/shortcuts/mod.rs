#[cfg(target_os = "macos")]
mod hook;
mod parser;

#[cfg(target_os = "macos")]
use std::sync::Arc;

use std::{
    collections::HashMap,
    sync::Mutex,
};

use crate::{app_state::AppState, preferences::ShortcutAction, windows::toggle_window};

#[cfg(target_os = "windows")]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg(target_os = "windows")]
use parser::parse_shortcut;

#[cfg(target_os = "windows")]
pub type ShortcutMap = Mutex<HashMap<Shortcut, ShortcutAction>>;

#[cfg(target_os = "macos")]
pub type ShortcutMap = Arc<Mutex<HashMap<String, ShortcutAction>>>;

pub fn load_shortcuts(
    app: &tauri::App,
    app_state: AppState,
) -> Result<ShortcutMap, Box<dyn std::error::Error>> {
    let shortcuts = app_state.preferences.get_shortcuts()?;

    #[cfg(target_os = "windows")]
    {
        let mut map = HashMap::new();

        for shortcut in shortcuts {
            let tauri_shortcut = parse_shortcut(&shortcut.keys).ok_or("invalid shortcut")?;
            map.insert(tauri_shortcut.clone(), shortcut.action.clone());
            app.global_shortcut().register(tauri_shortcut)?;
        }

        Ok(Mutex::new(map))
    }

    #[cfg(target_os = "macos")]
    {
        let mut map = HashMap::new();

        for shortcut in shortcuts {
            map.insert(normalize_keys(&shortcut.keys), shortcut.action.clone());
        }

        let shared_map = Arc::new(Mutex::new(map));
        hook::start(app.handle().clone(), Arc::clone(&shared_map));

        Ok(shared_map)
    }
}

pub fn handle_action(app: &tauri::AppHandle, action: &ShortcutAction) {
    match action {
        ShortcutAction::OpenClipboardHistory => {
            toggle_window(app, "clipboard");
        }
        _ => {}
    }
}

#[cfg(target_os = "windows")]
pub fn reload_shortcuts(
    app: &tauri::AppHandle,
    map: &ShortcutMap,
    shortcuts: &[crate::preferences::Shortcut],
) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    let mut guard = map.lock().map_err(|e| e.to_string())?;
    guard.clear();

    for shortcut in shortcuts {
        let tauri_shortcut = parse_shortcut(&shortcut.keys)
            .ok_or_else(|| format!("invalid shortcut: {}", shortcut.keys))?;
        app.global_shortcut()
            .register(tauri_shortcut.clone())
            .map_err(|e| e.to_string())?;
        guard.insert(tauri_shortcut, shortcut.action.clone());
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn reload_shortcuts(
    map: &ShortcutMap,
    shortcuts: &[crate::preferences::Shortcut],
) -> Result<(), String> {
    let mut guard = map.lock().map_err(|e| e.to_string())?;
    guard.clear();
    for shortcut in shortcuts {
        guard.insert(normalize_keys(&shortcut.keys), shortcut.action.clone());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn normalize_keys(keys: &str) -> String {
    keys.split('+')
        .map(|p| p.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join("+")
}
