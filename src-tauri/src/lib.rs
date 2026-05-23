mod commands;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            commands::start_metrics_loop(app.handle().clone());

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
                                window.hide().ok();
                            } else {
                                close_pending.store(false, Ordering::Relaxed);
                                is_open.store(true, Ordering::Relaxed);

                                window.move_window(Position::TrayCenter).ok();
                                window.show().ok();
                                window.set_focus().ok();
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
                                window.hide().ok();
                                is_open.store(false, Ordering::Relaxed);
                                close_pending.store(false, Ordering::Relaxed);
                            }
                        });
                    }

                    _ => {}
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running app");
}