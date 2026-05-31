mod parser;

use std::collections::HashMap;

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{
    app_state::AppState, preferences::ShortcutAction, shortcuts::parser::parse_shortcut,
    windows::open_window,
};

pub type ShortcutMap = HashMap<Shortcut, ShortcutAction>;

pub fn load_shortcuts(
    app: &tauri::App,
    app_state: AppState,
) -> Result<ShortcutMap, Box<dyn std::error::Error>> {
    let mut shortcut_map = HashMap::new();
    let shortcuts = app_state.preferences.get_shortcuts()?;

    for shortcut in shortcuts {
        let tauri_shortcut = parse_shortcut(&shortcut.keys).ok_or("invalid shortcut")?;

        shortcut_map.insert(tauri_shortcut.clone(), shortcut.action.clone());

        app.global_shortcut().register(tauri_shortcut)?;
    }

    Ok(shortcut_map)
}

pub fn handle_shortcuts(app: &tauri::AppHandle, action: &ShortcutAction) {
    match action {
        ShortcutAction::OpenClipboardHistory => {
            open_window(app, "clipboard");
        }
        _ => {}
    }
}
