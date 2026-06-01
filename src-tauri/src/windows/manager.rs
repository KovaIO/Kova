use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

pub fn open_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        window.show().ok();
        window.set_focus().ok();
    }
}

// pub fn hide_window(app: &AppHandle, label: &str) {
//     if let Some(window) = app.get_webview_window(label) {
//         window.hide().ok();
//     }
// }

pub fn toggle_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            window.show().ok();
            window.set_focus().ok();
        }
    }
}

/// Attach focus-loss auto-hide behavior to any window.
pub fn attach_focus_hide(window: WebviewWindow) {
    let is_open = Arc::new(AtomicBool::new(false));
    let close_pending = Arc::new(AtomicBool::new(false));

    window.on_window_event({
        let window = window.clone();
        let is_open = is_open.clone();
        let close_pending = close_pending.clone();

        move |event| match event {
            WindowEvent::Focused(true) => {
                is_open.store(true, Ordering::Relaxed);
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
}
