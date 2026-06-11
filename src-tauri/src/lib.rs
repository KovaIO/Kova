mod app_state;
mod apps;
mod clipboard;
mod commands;
mod disk;
mod license;
mod metrics;
mod migration;
mod preferences;
mod processes;
mod shortcuts;
mod update;
mod windows;
mod workspaces;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::WindowEvent;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};

use commands::{
    activate_license, apply_workspace, clear_clipboard_history, copy_clipboard_item,
    delete_all_disk_items, delete_clipboard_item, delete_disk_items, delete_workspace_profile,
    force_quit_process_cmd, get_apps, get_clipboard_history, get_current_metrics,
    get_disk_item_detail, get_disk_scan_preview, get_disk_scan_result, get_disk_volume_info,
    get_license, get_portal_url, get_preferences, get_process_history, get_snapshot,
    get_workspace_profile, get_workspace_profiles, open_clipboard_url, open_monitor,
    open_preferences, open_process, paste_clipboard_item, paste_plain_clipboard_item,
    preview_clipboard_item, quit_process_cmd, reveal_clipboard_item, save_workspace_profile,
    start_disk_scan, update_appearance_preferences, update_clipboard_preferences,
    update_general_preferences, update_shortcuts,
};

use clipboard::start_clipboard_watcher;

use crate::{
    app_state::initialize_app_state,
    metrics::{models::new_shared_history, start_metrics_loop},
    shortcuts::{handle_action, load_shortcuts, ShortcutMap},
    update::{check_for_updates, dismiss_update, get_update, install_update, UpdateState},
};

struct IsOpen(Arc<AtomicBool>);
struct AppTrayIcon(TrayIcon);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostarted"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
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
            get_portal_url,
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
            get_disk_volume_info,
            get_disk_scan_preview,
            start_disk_scan,
            get_disk_scan_result,
            get_disk_item_detail,
            delete_disk_items,
            delete_all_disk_items,
            open_preferences,
            open_monitor,
            open_process,
            get_update,
            dismiss_update,
            install_update
        ])
        .setup(|app| {
            let app_state = initialize_app_state(app)?;

            app.manage(app_state.clone());

            if let Ok(prefs) = app_state.preferences.get_preferences() {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                if prefs.general.launch_at_startup {
                    let _ = autostart.enable();
                } else {
                    let _ = autostart.disable();
                }
            }

            let app_handle = app.handle().clone();
            let license_service = app_state.license.clone();

            tauri::async_runtime::spawn(async move {
                match license_service.verify_if_needed().await {
                    Ok(Some(info)) => {
                        app_handle.emit("license-updated", &info).ok();
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("license verification failed: {}", err);
                    }
                }
            });

            app.manage(UpdateState::new());

            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                check_for_updates(update_handle).await;
            });

            let shortcut_map = load_shortcuts(app, app_state.clone())?;

            start_clipboard_watcher(app.handle().clone(), app_state);

            app.manage(shortcut_map);

            let history = new_shared_history();
            start_metrics_loop(app.handle().clone(), history.clone());

            app.manage(history);

            let is_open = Arc::new(AtomicBool::new(false));
            let close_pending = Arc::new(AtomicBool::new(false));

            app.manage(IsOpen(is_open.clone()));

            let tray = TrayIconBuilder::new()
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

            app.manage(AppTrayIcon(tray));

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
            if let Some(clippy) = app.get_webview_window("profiles") {
                windows::attach_focus_hide(clippy);
            }
            if let Some(update) = app.get_webview_window("update") {
                windows::attach_focus_hide(update);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running app");
}
