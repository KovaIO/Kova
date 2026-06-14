use std::sync::Arc;
use tauri::AppHandle;

use super::{handle_action, normalize_keys, ShortcutMap};

pub fn start(app: AppHandle, map: ShortcutMap) {
    std::thread::Builder::new()
        .name("ll-keyboard-hook".into())
        .spawn(move || macos_hook(app, map))
        .expect("failed to spawn hook thread");
}

fn macos_hook(app: AppHandle, map: ShortcutMap) {
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
        CallbackResult,
    };

    struct TapState {
        app: AppHandle,
        map: ShortcutMap,
    }

    let state = Arc::new(TapState { app, map });

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        move |_proxy, _event_type, event| {
            if event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_AUTOREPEAT,
            ) != 0
            {
                return CallbackResult::Keep;
            }

            let key_str = cg_event_to_key_string(event);
            if key_str.is_empty() {
                return CallbackResult::Keep;
            }
            if let Ok(guard) = state.map.lock() {
                if let Some(action) = guard.get(&key_str) {
                    handle_action(&state.app, action);
                    return CallbackResult::Drop;
                }
            }
            CallbackResult::Keep
        },
    )
    .expect("CGEventTap::new failed — grant Accessibility permission in System Settings");

    let source = tap
        .mach_port()
        .create_runloop_source(0)
        .expect("failed to create runloop source");

    let rl = CFRunLoop::get_current();
    rl.add_source(&source, unsafe { kCFRunLoopCommonModes });
    tap.enable();
    CFRunLoop::run_current();
}

fn cg_event_to_key_string(event: &core_graphics::event::CGEvent) -> String {
    use core_graphics::event::{CGEventFlags, EventField};

    let flags = event.get_flags();
    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;

    let ctrl = flags.contains(CGEventFlags::CGEventFlagControl);
    let alt = flags.contains(CGEventFlags::CGEventFlagAlternate);
    let shift = flags.contains(CGEventFlags::CGEventFlagShift);
    let cmd = flags.contains(CGEventFlags::CGEventFlagCommand);

    let key_name = keycode_to_name(keycode);
    if key_name.is_empty() {
        return String::new();
    }
    let mut parts: Vec<&str> = Vec::new();
    if ctrl {
        parts.push("ctrl");
    }
    if alt {
        parts.push("alt");
    }
    if shift {
        parts.push("shift");
    }
    if cmd {
        parts.push("super");
    }
    parts.push(&key_name);
    normalize_keys(&parts.join("+"))
}

fn keycode_to_name(code: u16) -> String {
    let name = match code {
        0x00 => "a",
        0x01 => "s",
        0x02 => "d",
        0x03 => "f",
        0x04 => "h",
        0x05 => "g",
        0x06 => "z",
        0x07 => "x",
        0x08 => "c",
        0x09 => "v",
        0x0B => "b",
        0x0C => "q",
        0x0D => "w",
        0x0E => "e",
        0x0F => "r",
        0x10 => "y",
        0x11 => "t",
        0x12 => "1",
        0x13 => "2",
        0x14 => "3",
        0x15 => "4",
        0x16 => "6",
        0x17 => "5",
        0x18 => "=",
        0x19 => "9",
        0x1A => "7",
        0x1B => "-",
        0x1C => "8",
        0x1D => "0",
        0x1E => "]",
        0x1F => "o",
        0x20 => "u",
        0x21 => "[",
        0x22 => "i",
        0x23 => "p",
        0x24 => "return",
        0x25 => "l",
        0x26 => "j",
        0x27 => "'",
        0x28 => "k",
        0x29 => ";",
        0x2A => "\\",
        0x2B => ",",
        0x2C => "/",
        0x2D => "n",
        0x2E => "m",
        0x2F => ".",
        0x30 => "tab",
        0x31 => "space",
        0x32 => "`",
        0x33 => "delete",
        0x35 => "escape",
        0x7B => "left",
        0x7C => "right",
        0x7D => "down",
        0x7E => "up",
        0x60 => "f5",
        0x61 => "f6",
        0x62 => "f7",
        0x63 => "f3",
        0x64 => "f8",
        0x65 => "f9",
        0x67 => "f11",
        0x6B => "f14",
        0x6D => "f10",
        0x6F => "f12",
        0x76 => "f4",
        0x78 => "f2",
        0x7A => "f1",
        _ => return String::new(),
    };
    name.to_string()
}
