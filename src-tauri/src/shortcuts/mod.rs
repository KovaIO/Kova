#[cfg(target_os = "macos")]
mod hook;
mod parser;

#[cfg(target_os = "macos")]
use std::sync::Arc;

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Mutex},
};

use crate::{
    app_state::AppState,
    license::LicenseTier,
    preferences::ShortcutAction,
    windows::{hide_window, open_window, toggle_window},
    AppTrayIcon, IsOpen,
};

use tauri::Manager;
#[cfg(target_os = "windows")]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg(target_os = "windows")]
use parser::parse_shortcut;
use tauri_plugin_positioner::{Position, WindowExt};

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
    let state = app.state::<AppState>();
    match action {
        ShortcutAction::OpenClipboardHistory => {
            toggle_window(app, "clipboard");
        }
        ShortcutAction::ApplyWorkspace => {
            let tier = match state.license.tier() {
                Ok(t) => t,
                Err(_) => return,
            };
            if tier != LicenseTier::Pro {
                return;
            }

            let app_handle = app.clone();

            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<crate::app_state::AppState>();

                let profiles = match state.workspaces.get_profiles() {
                    Ok(p) => p,
                    Err(_) => return,
                };

                if profiles.is_empty() {
                    return;
                }

                if profiles.len() == 1 {
                    let _ = state
                        .workspaces
                        .apply_profile(&profiles[0].id, &app_handle)
                        .await;
                } else {
                    toggle_window(&app_handle, "profiles");
                }
            });
        }
        ShortcutAction::OpenMonitor => {
            toggle_window(app, "monitor");
        }
        ShortcutAction::OpenMenubarPopover => {
            let window = app.get_webview_window("home").unwrap();
            let is_open = app.state::<IsOpen>();
            if is_open.0.load(Ordering::Relaxed) {
                hide_window(&window);
                is_open.0.store(false, Ordering::Relaxed);
            } else {
                let tray = app.state::<AppTrayIcon>();
                if let Ok(Some(rect)) = tray.0.rect() {
                    let size = window.outer_size().unwrap_or_default();
                    let tauri::Position::Physical(pos) = rect.position else {
                        return;
                    };
                    let tauri::Size::Physical(s) = rect.size else {
                        return;
                    };
                    let x = pos.x + s.width as i32 / 2 - size.width as i32 / 2;
                    let y = pos.y - size.height as i32;
                    window.set_position(tauri::PhysicalPosition { x, y }).ok();
                } else {
                    window.move_window(Position::TopRight).ok();
                }
                open_window(app, "home");
                is_open.0.store(true, Ordering::Relaxed);
            }
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
