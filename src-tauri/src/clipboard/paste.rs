#[cfg(target_os = "windows")]
pub fn simulate_paste() {
    use std::mem::size_of;
    use winapi::um::winuser::{SendInput, INPUT, KEYEVENTF_KEYUP, VK_CONTROL};

    let vk_v: u16 = 0x56; // V key

    let inputs: [INPUT; 4] = [
        // Ctrl down
        make_key_input(VK_CONTROL as u16, 0),
        // V down
        make_key_input(vk_v, 0),
        // V up
        make_key_input(vk_v, KEYEVENTF_KEYUP),
        // Ctrl up
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

    let cmd_v: CGKeyCode = 0x09; // 'V' key

    // Cmd+V down
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), cmd_v, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }

    // Cmd+V up
    if let Ok(event) = CGEvent::new_keyboard_event(source, cmd_v, false) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }
}
