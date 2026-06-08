use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WindowEvent};

pub fn open_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        window.show().ok();
        let _ = window.emit_to(label, "window-opened", ());
        window.set_focus().ok();
    }
}

pub fn hide_window(window: &WebviewWindow) {
    let label = window.label();
    let _ = window.emit_to(label, "window-closed", ());
    window.hide().ok();
}

pub fn toggle_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if window.is_visible().unwrap_or(false) {
            hide_window(&window);
        } else {
            open_window(app, label);
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
                        hide_window(&window);
                        is_open.store(false, Ordering::Relaxed);
                        close_pending.store(false, Ordering::Relaxed);
                    }
                });
            }

            _ => {}
        }
    });
}

#[derive(Debug)]
pub struct WindowRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn resolve_rect(app: &AppHandle, fx: f32, fy: f32, fw: f32, fh: f32, gap: u32) -> Option<WindowRect> {
    let window = app.get_webview_window("home")?;
    let monitor = window.primary_monitor().ok()??;

    let area = monitor.work_area();
    let sw = area.size.width as f64;
    let sh = area.size.height as f64;
    let g = gap as f64;

    Some(WindowRect {
        x: area.position.x as f64 + fx as f64 * sw + g,
        y: area.position.y as f64 + fy as f64 * sh + g,
        width: fw as f64 * sw - g * 2.0,
        height: fh as f64 * sh - g * 2.0,
    })
}
