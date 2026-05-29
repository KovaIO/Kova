use tauri::State;

use crate::{
    app_state::AppState,
    preferences::{
        models::{ClipboardPreferences, GeneralPreferences, Preferences},
        WindowManagerPreferences,
    },
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
        state
            .preferences
            .update_general_preferences(prefs)
            .map_err(|e| e.to_string())
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
pub fn update_window_manager_preferences(
    prefs: WindowManagerPreferences,
    state: State<AppState>,
) -> Result<(), String> {
    with_emit(&state, || {
        state
            .preferences
            .update_window_manager_preferences(prefs)
            .map_err(|e| e.to_string())
    })
}
