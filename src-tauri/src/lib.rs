mod app_state;
mod clipboard;
mod commands;
mod license;
mod metrics;
mod migration;
mod preferences;
mod processes;
mod shortcuts;
mod windows;
mod workspaces;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

use commands::{
    activate_license, apply_workspace, clear_clipboard_history, copy_clipboard_item,
    delete_clipboard_item, delete_workspace_profile, force_quit_process_cmd, get_apps,
    get_clipboard_history, get_current_metrics, get_license, get_preferences, get_process_history,
    get_snapshot, get_workspace_profile, get_workspace_profiles, open_clipboard_url, open_monitor,
    open_preferences, open_process, paste_clipboard_item, paste_plain_clipboard_item,
    preview_clipboard_item, quit_process_cmd, reveal_clipboard_item, save_workspace_profile,
    update_appearance_preferences, update_clipboard_preferences, update_general_preferences,
    update_shortcuts,
};

use clipboard::start_clipboard_watcher;

use crate::{
    app_state::initialize_app_state,
    metrics::{models::new_shared_history, start_metrics_loop},
    shortcuts::{handle_action, load_shortcuts, ShortcutMap},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;

                    if event.state() != ShortcutState::Pressed {
                        return;
                    }

                    let map = app.state::<ShortcutMap>();
                    let Ok(guard) = map.lock() else {
                        return;
                    };

                    if let Some(action) = guard.get(&shortcut) {
                        handle_action(app, action);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_preferences,
            get_license,
            activate_license,
            update_general_preferences,
            update_clipboard_preferences,
            update_appearance_preferences,
            update_shortcuts,
            get_current_metrics,
            get_snapshot,
            get_process_history,
            get_apps,
            quit_process_cmd,
            force_quit_process_cmd,
            get_clipboard_history,
            paste_clipboard_item,
            copy_clipboard_item,
            paste_plain_clipboard_item,
            open_clipboard_url,
            reveal_clipboard_item,
            preview_clipboard_item,
            delete_clipboard_item,
            clear_clipboard_history,
            apply_workspace,
            get_workspace_profile,
            get_workspace_profiles,
            save_workspace_profile,
            delete_workspace_profile,
            open_preferences,
            open_monitor,
            open_process
        ])
        .setup(|app| {
            let app_state = initialize_app_state(app)?;

            app.manage(app_state.clone());

            let shortcut_map = load_shortcuts(app, app_state.clone())?;

            start_clipboard_watcher(app.handle().clone(), app_state);

            app.manage(shortcut_map);

            let history = new_shared_history();
            start_metrics_loop(app.handle().clone(), history.clone());

            app.manage(history);

            let is_open = Arc::new(AtomicBool::new(false));
            let close_pending = Arc::new(AtomicBool::new(false));

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event({
                    let is_open = is_open.clone();
                    let close_pending = close_pending.clone();

                    move |tray, event| {
                        tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let window = tray.app_handle().get_webview_window("home").unwrap();

                            if is_open.load(Ordering::Relaxed) {
                                close_pending.store(false, Ordering::Relaxed);
                                is_open.store(false, Ordering::Relaxed);
                                windows::hide_window(&window);
                            } else {
                                close_pending.store(false, Ordering::Relaxed);
                                is_open.store(true, Ordering::Relaxed);

                                window.move_window(Position::TrayCenter).ok();
                                windows::open_window(tray.app_handle(), "home");
                            }
                        }
                    }
                })
                .build(app)?;

            let window = app.get_webview_window("home").unwrap();

            window.on_window_event({
                let window = window.clone();
                let is_open = is_open.clone();
                let close_pending = close_pending.clone();

                move |event| match event {
                    WindowEvent::Focused(true) => {
                        close_pending.store(false, Ordering::Relaxed);
                    }

                    WindowEvent::Focused(false) => {
                        if !is_open.load(Ordering::Relaxed) {
                            return;
                        }

                        close_pending.store(true, Ordering::Relaxed);

                        let window = window.clone();
                        let close_pending = close_pending.clone();
                        let is_open = is_open.clone();

                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                            if close_pending.load(Ordering::Relaxed)
                                && !window.is_focused().unwrap_or(false)
                            {
                                windows::hide_window(&window);
                                is_open.store(false, Ordering::Relaxed);
                                close_pending.store(false, Ordering::Relaxed);
                            }
                        });
                    }

                    _ => {}
                }
            });

            if let Some(monitor) = app.get_webview_window("monitor") {
                windows::attach_focus_hide(monitor);
            }
            if let Some(process) = app.get_webview_window("process") {
                windows::attach_focus_hide(process);
            }

            if let Some(clippy) = app.get_webview_window("clipboard") {
                windows::attach_focus_hide(clippy);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running app");
}
