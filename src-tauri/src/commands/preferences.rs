use tauri::State;

use crate::{
    app_state::AppState,
    preferences::{
        models::{ClipboardPreferences, GeneralPreferences, Preferences},
        AppearancePreferences, Shortcut,
    },
    shortcuts::ShortcutMap,
};

fn with_emit<T, F>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let result = f()?;

    state.emit_preferences_updated();

    Ok(result)
}

#[tauri::command]
pub fn get_preferences(state: State<AppState>) -> Result<Preferences, String> {
    state
        .preferences
        .get_preferences()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_general_preferences(
    prefs: GeneralPreferences,
    state: State<AppState>,
) -> Result<(), String> {
    with_emit(&state, || {
        state.preferences.update_general_preferences(prefs)
    })
}

#[tauri::command]
pub fn update_clipboard_preferences(
    prefs: ClipboardPreferences,
    state: State<AppState>,
) -> Result<(), String> {
    with_emit(&state, || {
        state
            .preferences
            .update_clipboard_preferences(prefs)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn update_shortcuts(
    shortcuts: Vec<Shortcut>,
    state: State<AppState>,
    app: tauri::AppHandle,
    map: State<ShortcutMap>,
) -> Result<(), String> {
    with_emit(&state, || {
        state.preferences.update_shortcuts(shortcuts.clone())?;

        #[cfg(target_os = "windows")]
        crate::shortcuts::reload_shortcuts(&app, &map, &shortcuts)?;

        #[cfg(target_os = "macos")]
        crate::shortcuts::reload_shortcuts(&map, &shortcuts)?;

        Ok(())
    })
}

#[tauri::command]
pub fn update_appearance_preferences(
    prefs: AppearancePreferences,
    state: State<AppState>,
) -> Result<(), String> {
    with_emit(&state, || {
        state.preferences.update_appearance_preferences(prefs)
    })
}
