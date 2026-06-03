#[cfg(target_os = "windows")]
pub fn simulate_paste() {
    use std::mem::size_of;
    use winapi::um::winuser::{SendInput, INPUT, KEYEVENTF_KEYUP, VK_CONTROL};

    let vk_v: u16 = 0x56;

    let inputs: [INPUT; 4] = [
        make_key_input(VK_CONTROL as u16, 0),
        make_key_input(vk_v, 0),
        make_key_input(vk_v, KEYEVENTF_KEYUP),
        make_key_input(VK_CONTROL as u16, KEYEVENTF_KEYUP),
    ];

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr() as *mut INPUT,
            size_of::<INPUT>() as i32,
        );
    }
}

#[cfg(target_os = "windows")]
fn make_key_input(vk: u16, flags: u32) -> winapi::um::winuser::INPUT {
    use winapi::um::winuser::{INPUT, INPUT_KEYBOARD, KEYBDINPUT};
    let mut input = unsafe { std::mem::zeroed::<INPUT>() };
    input.type_ = INPUT_KEYBOARD;
    unsafe {
        let ki = input.u.ki_mut();
        *ki = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
    }
    input
}

#[cfg(target_os = "macos")]
pub fn simulate_paste() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => return,
    };

    let cmd_v: CGKeyCode = 0x09;

    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), cmd_v, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }
    if let Ok(event) = CGEvent::new_keyboard_event(source, cmd_v, false) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }
}

#[cfg(target_os = "windows")]
thread_local! {
    static IMAGE_BGR: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
    static IMAGE_SIZE: std::cell::RefCell<(u32, u32)> = std::cell::RefCell::new((0, 0));
}

#[cfg(target_os = "windows")]
pub fn set_image_to_clipboard_delayed(path: String) -> Result<String, String> {
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage,
        CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, MSG, WNDCLASSW, WS_OVERLAPPED,
    };
    use windows::core::PCWSTR;

    const CF_DIB: u32 = 8;

    // Decode image BEFORE spawning — do the heavy work on the calling thread
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let signature = format!("{}x{}:{}", width, height, rgba.len());

    // Pre-convert to BGR so wnd_proc just needs to copy memory
    let mut bgr: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        bgr.push(pixel[2]);
        bgr.push(pixel[1]);
        bgr.push(pixel[0]);
        bgr.push(pixel[3]);
    }

    std::thread::spawn(move || unsafe {
        let class_name: Vec<u16> = "KovaClipboard\0".encode_utf16().collect();

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(clipboard_wnd_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        // Ignore error — class may already be registered from a previous call
        let _ = RegisterClassW(&wc);

        let hwnd = match CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WS_OVERLAPPED,
            CW_USEDEFAULT, CW_USEDEFAULT, 0, 0,
            None, None, None, None,
        ) {
            Ok(h) => h,
            Err(e) => { eprintln!("CreateWindowExW failed: {e}"); return; }
        };

        // Store pre-decoded BGR data in thread-locals
        IMAGE_BGR.with(|b| *b.borrow_mut() = bgr);
        IMAGE_SIZE.with(|s| *s.borrow_mut() = (width, height));

        if OpenClipboard(Some(hwnd)).is_ok() {
            let _ = EmptyClipboard();
            let _ = SetClipboardData(CF_DIB, None); // NULL = delayed rendering promise
            let _ = CloseClipboard();
        }

        // Message loop — exits when WM_DESTROYCLIPBOARD fires
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, Some(hwnd), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up thread-locals after loop exits
        IMAGE_BGR.with(|b| b.borrow_mut().clear());
    });

    Ok(signature)
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn clipboard_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Graphics::Gdi::{BITMAPINFOHEADER, BI_RGB};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, DestroyWindow, PostQuitMessage,
        WM_DESTROYCLIPBOARD, WM_RENDERALLFORMATS, WM_RENDERFORMAT,
    };

    const CF_DIB: u32 = 8;

    match msg {
        m if m == WM_RENDERFORMAT || m == WM_RENDERALLFORMATS => {
            let bgr = IMAGE_BGR.with(|b| b.borrow().clone());
            let (width, height) = IMAGE_SIZE.with(|s| *s.borrow());

            if bgr.is_empty() {
                return windows::Win32::Foundation::LRESULT(0);
            }

            let header_size = std::mem::size_of::<BITMAPINFOHEADER>();
            let total = header_size + bgr.len();

            if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, total) {
                let ptr = GlobalLock(hmem) as *mut u8;
                if !ptr.is_null() {
                    let header = BITMAPINFOHEADER {
                        biSize: header_size as u32,
                        biWidth: width as i32,
                        biHeight: -(height as i32),
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        biSizeImage: bgr.len() as u32,
                        biXPelsPerMeter: 0,
                        biYPelsPerMeter: 0,
                        biClrUsed: 0,
                        biClrImportant: 0,
                    };
                    std::ptr::copy_nonoverlapping(
                        &header as *const _ as *const u8,
                        ptr,
                        header_size,
                    );
                    std::ptr::copy_nonoverlapping(
                        bgr.as_ptr(),
                        ptr.add(header_size),
                        bgr.len(),
                    );
                    let _ = GlobalUnlock(hmem);

                    if m == WM_RENDERALLFORMATS {
                        let _ = OpenClipboard(Some(hwnd));
                        let _ = EmptyClipboard();
                    }
                    let _ = SetClipboardData(CF_DIB, Some(HANDLE(hmem.0)));
                    if m == WM_RENDERALLFORMATS {
                        let _ = CloseClipboard();
                    }
                }
            }
            windows::Win32::Foundation::LRESULT(0)
        }

        // Another app took ownership — release and exit
        m if m == WM_DESTROYCLIPBOARD => {
            let _ = DestroyWindow(hwnd);
            PostQuitMessage(0);
            windows::Win32::Foundation::LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}