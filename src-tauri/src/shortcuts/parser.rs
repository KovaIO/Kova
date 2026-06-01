use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub fn parse_shortcut(keys: &str) -> Option<Shortcut> {
    let parts: Vec<_> = keys.split('+').map(|s| s.trim()).collect();

    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => {
                modifiers |= Modifiers::CONTROL;
            }

            "shift" => {
                modifiers |= Modifiers::SHIFT;
            }

            "alt" => {
                modifiers |= Modifiers::ALT;
            }

            "super" | "cmd" | "command" => {
                modifiers |= Modifiers::SUPER;
            }

            "tab" => {
                code = Some(Code::Tab);
            }

            "space" => {
                code = Some(Code::Space);
            }

            "enter" | "return" => {
                code = Some(Code::Enter);
            }

            "up" | "↑" => {
                code = Some(Code::ArrowUp);
            }

            "down" | "↓" => {
                code = Some(Code::ArrowDown);
            }

            "left" | "←" => {
                code = Some(Code::ArrowLeft);
            }

            "right" | "→" => {
                code = Some(Code::ArrowRight);
            }

            "`" | "~" | "backquote" => {
                code = Some(Code::Backquote);
            }

            "-" | "minus" => {
                code = Some(Code::Minus);
            }

            "=" | "equal" | "plus" => {
                code = Some(Code::Equal);
            }

            key if key.len() == 1 => {
                let ch = key.chars().next()?.to_ascii_uppercase();

                code = Some(match ch {
                    'A' => Code::KeyA,
                    'B' => Code::KeyB,
                    'C' => Code::KeyC,
                    'D' => Code::KeyD,
                    'E' => Code::KeyE,
                    'F' => Code::KeyF,
                    'G' => Code::KeyG,
                    'H' => Code::KeyH,
                    'I' => Code::KeyI,
                    'J' => Code::KeyJ,
                    'K' => Code::KeyK,
                    'L' => Code::KeyL,
                    'M' => Code::KeyM,
                    'N' => Code::KeyN,
                    'O' => Code::KeyO,
                    'P' => Code::KeyP,
                    'Q' => Code::KeyQ,
                    'R' => Code::KeyR,
                    'S' => Code::KeyS,
                    'T' => Code::KeyT,
                    'U' => Code::KeyU,
                    'V' => Code::KeyV,
                    'W' => Code::KeyW,
                    'X' => Code::KeyX,
                    'Y' => Code::KeyY,
                    'Z' => Code::KeyZ,

                    '0' => Code::Digit0,
                    '1' => Code::Digit1,
                    '2' => Code::Digit2,
                    '3' => Code::Digit3,
                    '4' => Code::Digit4,
                    '5' => Code::Digit5,
                    '6' => Code::Digit6,
                    '7' => Code::Digit7,
                    '8' => Code::Digit8,
                    '9' => Code::Digit9,

                    _ => return None,
                });
            }

            _ => return None,
        }
    }

    let code = code?;

    if modifiers.is_empty() {
        Some(Shortcut::new(None, code))
    } else {
        Some(Shortcut::new(Some(modifiers), code))
    }
}
