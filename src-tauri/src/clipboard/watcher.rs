use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use arboard::{Clipboard, ImageData};
use image::ImageEncoder;
use tauri::{AppHandle, Emitter};

use crate::{
    app_state::AppState,
    clipboard::{
        models::{ClipboardContentType, SourceApp},
        source::{get_foreground_app, is_app_ignored},
    },
};

static WATCHER_STARTED: AtomicBool = AtomicBool::new(false);

static LAST_CAPTURE: Mutex<LastCapture> = Mutex::new(LastCapture {
    text: None,
    image_signature: None,
});

#[derive(Default)]
struct LastCapture {
    text: Option<String>,
    image_signature: Option<String>,
}

pub fn remember_text(text: String) {
    if let Ok(mut g) = LAST_CAPTURE.lock() {
        g.text = Some(text);
        g.image_signature = None;
    }
}

pub fn remember_image_signature(sig: String) {
    if let Ok(mut g) = LAST_CAPTURE.lock() {
        g.image_signature = Some(sig);
        g.text = None;
    }
}

pub fn start(app: AppHandle, state: AppState) {
    if WATCHER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(target_os = "windows")]
    start_windows(app, state);

    #[cfg(not(target_os = "windows"))]
    start_polling(app, state);
}

#[cfg(target_os = "windows")]
fn start_windows(app: AppHandle, state: AppState) {
    // Shared between the message-loop thread and the worker thread
    struct PendingCapture {
        source: Option<SourceApp>,
    }

    static PENDING: Mutex<Option<PendingCapture>> = Mutex::new(None);

    // Worker thread: processes captures off the message loop
    let app_worker = app.clone();
    let state_worker = state.clone();
    std::thread::Builder::new()
        .name("clipboard-worker".into())
        .spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("clipboard worker: {e}");
                    return;
                }
            };

            loop {
                // Check for pending capture
                let pending = {
                    let mut guard = PENDING.lock().unwrap();
                    guard.take()
                };

                if let Some(capture) = pending {
                    let prefs = match state_worker.preferences.get_preferences() {
                        Ok(p) => p,
                        Err(_) => {
                            std::thread::sleep(Duration::from_millis(50));
                            continue;
                        }
                    };
                    if prefs.clipboard.enabled {
                        if let Err(e) = process_clipboard(
                            &app_worker,
                            &state_worker,
                            &mut clipboard,
                            capture.source,
                            &prefs,
                        ) {
                            eprintln!("clipboard worker: {e}");
                        }
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(30));
                }
            }
        })
        .expect("failed to spawn clipboard-worker");

    // Message-loop thread: owns the hidden window, receives WM_CLIPBOARDUPDATE
    std::thread::Builder::new()
        .name("clipboard-listener".into())
        .spawn(move || unsafe {
            use windows::core::PCWSTR;
            use windows::Win32::System::DataExchange::AddClipboardFormatListener;
            use windows::Win32::UI::WindowsAndMessaging::{
                CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage,
                CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WNDCLASSW, WS_OVERLAPPED,
            };

            unsafe extern "system" fn wnd_proc(
                hwnd: windows::Win32::Foundation::HWND,
                msg: u32,
                wparam: windows::Win32::Foundation::WPARAM,
                lparam: windows::Win32::Foundation::LPARAM,
            ) -> windows::Win32::Foundation::LRESULT {
                use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_CLIPBOARDUPDATE};

                if msg == WM_CLIPBOARDUPDATE {
                    // Snapshot foreground app immediately — this fires the
                    // instant the clipboard changes, before any app switch
                    let source = get_foreground_app();

                    if let Ok(mut guard) = PENDING.lock() {
                        *guard = Some(PendingCapture { source });
                    }
                }

                DefWindowProcW(hwnd, msg, wparam, lparam)
            }

            let class_name: Vec<u16> = "KovaWatcher\0".encode_utf16().collect();
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let hwnd = match CreateWindowExW(
                Default::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                None,
                None,
                None,
                None,
            ) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("clipboard listener: CreateWindowExW: {e}");
                    return;
                }
            };

            if let Err(e) = AddClipboardFormatListener(hwnd) {
                eprintln!("clipboard listener: AddClipboardFormatListener: {e}");
                return;
            }

            eprintln!("[watcher] listening for WM_CLIPBOARDUPDATE");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })
        .expect("failed to spawn clipboard-listener");
}

#[cfg(target_os = "windows")]
fn process_clipboard(
    app: &AppHandle,
    state: &AppState,
    clipboard: &mut Clipboard,
    source: Option<SourceApp>,
    prefs: &crate::preferences::models::Preferences,
) -> Result<(), String> {
    // Filter our own process before doing any clipboard reads
    if let Some(ref s) = source {
        if is_own_process(s) || is_app_ignored(s, &prefs.clipboard.ignored_apps) {
            return Ok(());
        }
    }

    if let Ok(image) = clipboard.get_image() {
        let signature = format!("{}x{}:{}", image.width, image.height, image.bytes.len());
        let is_known = {
            let g = LAST_CAPTURE.lock().map_err(|e| e.to_string())?;
            g.image_signature.as_ref() == Some(&signature)
        };
        if !is_known {
            try_capture_image(app, state, &image, &signature, &source, prefs)?;
        }
        return Ok(());
    }

    if let Ok(text) = clipboard.get_text() {
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            try_capture_text(app, state, trimmed, &source, prefs)?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn start_polling(app: AppHandle, state: AppState) {
    std::thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("clipboard watcher: {e}");
                return;
            }
        };

        loop {
            std::thread::sleep(Duration::from_millis(450));

            let prefs = match state.preferences.get_preferences() {
                Ok(p) => p,
                Err(_) => continue,
            };
            if !prefs.clipboard.enabled {
                continue;
            }

            // Snapshot source immediately at poll time
            let source = get_foreground_app();

            if let Err(e) = process_clipboard_poll(&app, &state, &mut clipboard, source, &prefs) {
                eprintln!("clipboard watcher: {e}");
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn process_clipboard_poll(
    app: &AppHandle,
    state: &AppState,
    clipboard: &mut Clipboard,
    source: Option<SourceApp>,
    prefs: &crate::preferences::models::Preferences,
) -> Result<(), String> {
    if let Some(ref s) = source {
        if is_own_process(s) || is_app_ignored(s, &prefs.clipboard.ignored_apps) {
            return Ok(());
        }
    }

    if let Ok(image) = clipboard.get_image() {
        let signature = format!("{}x{}:{}", image.width, image.height, image.bytes.len());
        let is_known = {
            let g = LAST_CAPTURE.lock().map_err(|e| e.to_string())?;
            g.image_signature.as_ref() == Some(&signature)
        };
        if !is_known {
            try_capture_image(app, state, &image, &signature, &source, prefs)?;
        }
        return Ok(());
    }

    if let Ok(text) = clipboard.get_text() {
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            try_capture_text(app, state, trimmed, &source, prefs)?;
        }
    }

    Ok(())
}

fn try_capture_text(
    app: &AppHandle,
    state: &AppState,
    text: String,
    source: &Option<SourceApp>,
    prefs: &crate::preferences::models::Preferences,
) -> Result<bool, String> {
    {
        let g = LAST_CAPTURE.lock().map_err(|e| e.to_string())?;
        if g.text.as_ref() == Some(&text) {
            return Ok(false);
        }
    }

    if prefs.clipboard.ignore_passwords && looks_like_password(&text) {
        return Ok(false);
    }

    if state
        .clipboard
        .latest_matches(ClipboardContentType::Text, Some(&text), None)?
    {
        return Ok(false);
    }

    let created_at = chrono_timestamp();
    state.clipboard.insert_item(
        ClipboardContentType::Text,
        Some(&text),
        None,
        source.as_ref().map(|s| s.name.as_str()),
        source
            .as_ref()
            .map(|s| s.path.as_str())
            .filter(|p| !p.is_empty()),
        created_at,
    )?;
    let removed = state
        .clipboard
        .trim_to_limit(prefs.clipboard.history_limit)?;
    cleanup_image_files(&removed);
    remember_text(text);
    emit_updated(app);
    Ok(true)
}

fn try_capture_image(
    app: &AppHandle,
    state: &AppState,
    image: &ImageData,
    signature: &str,
    source: &Option<SourceApp>,
    prefs: &crate::preferences::models::Preferences,
) -> Result<bool, String> {
    let path = save_image(state, image)?;

    if state
        .clipboard
        .latest_matches(ClipboardContentType::Image, None, Some(&path))?
    {
        let _ = std::fs::remove_file(&path);
        remember_image_signature(signature.to_string());
        return Ok(false);
    }

    let created_at = chrono_timestamp();
    state.clipboard.insert_item(
        ClipboardContentType::Image,
        None,
        Some(&path),
        source.as_ref().map(|s| s.name.as_str()),
        source
            .as_ref()
            .map(|s| s.path.as_str())
            .filter(|p| !p.is_empty()),
        created_at,
    )?;
    let removed = state
        .clipboard
        .trim_to_limit(prefs.clipboard.history_limit)?;
    cleanup_image_files(&removed);
    remember_image_signature(signature.to_string());
    emit_updated(app);
    Ok(true)
}

fn save_image(state: &AppState, image: &ImageData) -> Result<String, String> {
    let file_name = format!("{}.png", uuid_simple());
    let path = state.clipboard_images_dir.join(&file_name);
    let img = image::RgbaImage::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.to_vec(),
    )
    .ok_or_else(|| "invalid image dimensions".to_string())?;
    let mut buffer = Vec::with_capacity(img.len());
    image::codecs::png::PngEncoder::new_with_quality(
        &mut buffer,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    )
    .write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(&path, &buffer).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

fn is_own_process(source: &SourceApp) -> bool {
    if let Ok(exe) = std::env::current_exe() {
        let own = exe.to_string_lossy().to_lowercase();
        let cur = source.path.to_lowercase();
        if !cur.is_empty() && cur == own {
            return true;
        }
    }
    source.name.eq_ignore_ascii_case("kova") || source.name.eq_ignore_ascii_case("app")
}

fn looks_like_password(text: &str) -> bool {
    if text.len() > 128 {
        return false;
    }
    text.len() >= 12
        && text.chars().any(|c| c.is_uppercase())
        && text.chars().any(|c| c.is_lowercase())
        && text.chars().any(|c| c.is_ascii_digit())
        && text
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn uuid_simple() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

fn emit_updated(app: &AppHandle) {
    let _ = app.emit("clipboard-history-updated", ());
}

pub fn cleanup_image_files(paths: &[String]) {
    for p in paths {
        let _ = std::fs::remove_file(p);
    }
}

pub fn clear_history_files(state: &AppState) -> Result<(), String> {
    cleanup_image_files(&state.clipboard.clear_history()?);
    Ok(())
}

pub fn delete_item_files(state: &AppState, id: i64) -> Result<(), String> {
    if let Some(path) = state.clipboard.delete_item(id)? {
        cleanup_image_files(&[path]);
    }
    Ok(())
}
