use winit::{event::MouseButton, keyboard::KeyCode};

/// Translate winit physical keys into the LWJGL 2 key-code namespace used by
/// Minecraft 1.12.2 `Keyboard` and persisted by vanilla `options.txt`.
pub fn lwjgl_from_winit(key: KeyCode) -> Option<i32> {
    use KeyCode::*;
    Some(match key {
        Escape => 1,
        Digit1 => 2, Digit2 => 3, Digit3 => 4, Digit4 => 5, Digit5 => 6,
        Digit6 => 7, Digit7 => 8, Digit8 => 9, Digit9 => 10, Digit0 => 11,
        Minus => 12, Equal => 13, Backspace => 14, Tab => 15,
        KeyQ => 16, KeyW => 17, KeyE => 18, KeyR => 19, KeyT => 20,
        KeyY => 21, KeyU => 22, KeyI => 23, KeyO => 24, KeyP => 25,
        BracketLeft => 26, BracketRight => 27, Enter => 28, ControlLeft => 29,
        KeyA => 30, KeyS => 31, KeyD => 32, KeyF => 33, KeyG => 34,
        KeyH => 35, KeyJ => 36, KeyK => 37, KeyL => 38, Semicolon => 39,
        Quote => 40, Backquote => 41, ShiftLeft => 42, Backslash => 43,
        KeyZ => 44, KeyX => 45, KeyC => 46, KeyV => 47, KeyB => 48,
        KeyN => 49, KeyM => 50, Comma => 51, Period => 52, Slash => 53,
        ShiftRight => 54, NumpadMultiply => 55, AltLeft => 56, Space => 57,
        CapsLock => 58, F1 => 59, F2 => 60, F3 => 61, F4 => 62, F5 => 63,
        F6 => 64, F7 => 65, F8 => 66, F9 => 67, F10 => 68,
        NumLock => 69, ScrollLock => 70, Numpad7 => 71, Numpad8 => 72,
        Numpad9 => 73, NumpadSubtract => 74, Numpad4 => 75, Numpad5 => 76,
        Numpad6 => 77, NumpadAdd => 78, Numpad1 => 79, Numpad2 => 80,
        Numpad3 => 81, Numpad0 => 82, NumpadDecimal => 83,
        F11 => 87, F12 => 88, F13 => 100, F14 => 101, F15 => 102,
        NumpadEnter => 156, ControlRight => 157, NumpadDivide => 181,
        PrintScreen => 183, AltRight => 184, Pause => 197, Home => 199,
        ArrowUp => 200, PageUp => 201, ArrowLeft => 203, ArrowRight => 205,
        End => 207, ArrowDown => 208, PageDown => 209, Insert => 210,
        Delete => 211, SuperLeft => 219, SuperRight => 220, ContextMenu => 221,
        _ => return None,
    })
}

pub fn mouse_code(button: MouseButton) -> Option<i32> {
    Some(match button {
        MouseButton::Left => -100,
        MouseButton::Right => -99,
        MouseButton::Middle => -98,
        MouseButton::Back => -97,
        MouseButton::Forward => -96,
        MouseButton::Other(index) if index <= 100 => -100 + index as i32,
        MouseButton::Other(_) => return None,
    })
}


pub fn mouse_button_index(button: MouseButton) -> Option<i32> {
    Some(match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(index) if index <= i32::MAX as u16 => index as i32,
        MouseButton::Other(_) => return None,
    })
}

pub fn mouse_button_from_index(index: i32) -> Option<MouseButton> {
    Some(match index {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        value if (0..=u16::MAX as i32).contains(&value) => MouseButton::Other(value as u16),
        _ => return None,
    })
}

pub fn display_name(code: i32) -> String {
    if code < 0 {
        return match code {
            -100 => "Mouse 1".to_owned(),
            -99 => "Mouse 2".to_owned(),
            -98 => "Mouse 3".to_owned(),
            value => format!("Mouse {}", value + 101),
        };
    }
    if code >= 256 {
        return char::from_u32((code - 256) as u32).map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "?".to_owned());
    }
    match code {
        0 => "NONE", 1 => "ESCAPE", 2 => "1", 3 => "2", 4 => "3", 5 => "4", 6 => "5", 7 => "6", 8 => "7", 9 => "8", 10 => "9", 11 => "0",
        12 => "MINUS", 13 => "EQUALS", 14 => "BACK", 15 => "TAB", 16 => "Q", 17 => "W", 18 => "E", 19 => "R", 20 => "T", 21 => "Y", 22 => "U", 23 => "I", 24 => "O", 25 => "P",
        26 => "LBRACKET", 27 => "RBRACKET", 28 => "RETURN", 29 => "LCONTROL", 30 => "A", 31 => "S", 32 => "D", 33 => "F", 34 => "G", 35 => "H", 36 => "J", 37 => "K", 38 => "L", 39 => "SEMICOLON", 40 => "APOSTROPHE", 41 => "GRAVE", 42 => "LSHIFT", 43 => "BACKSLASH", 44 => "Z", 45 => "X", 46 => "C", 47 => "V", 48 => "B", 49 => "N", 50 => "M", 51 => "COMMA", 52 => "PERIOD", 53 => "SLASH", 54 => "RSHIFT", 55 => "MULTIPLY", 56 => "LMENU", 57 => "SPACE", 58 => "CAPITAL",
        59 => "F1", 60 => "F2", 61 => "F3", 62 => "F4", 63 => "F5", 64 => "F6", 65 => "F7", 66 => "F8", 67 => "F9", 68 => "F10", 69 => "NUMLOCK", 70 => "SCROLL", 71 => "NUMPAD7", 72 => "NUMPAD8", 73 => "NUMPAD9", 74 => "SUBTRACT", 75 => "NUMPAD4", 76 => "NUMPAD5", 77 => "NUMPAD6", 78 => "ADD", 79 => "NUMPAD1", 80 => "NUMPAD2", 81 => "NUMPAD3", 82 => "NUMPAD0", 83 => "DECIMAL", 87 => "F11", 88 => "F12", 100 => "F13", 101 => "F14", 102 => "F15", 156 => "NUMPADENTER", 157 => "RCONTROL", 181 => "DIVIDE", 183 => "SYSRQ", 184 => "RMENU", 197 => "PAUSE", 199 => "HOME", 200 => "UP", 201 => "PRIOR", 203 => "LEFT", 205 => "RIGHT", 207 => "END", 208 => "DOWN", 209 => "NEXT", 210 => "INSERT", 211 => "DELETE", 219 => "LMETA", 220 => "RMETA", 221 => "APPS",
        value => return format!("KEY{value}"),
    }.to_owned()
}

pub fn binding_matches_key(binding_code: i32, key: KeyCode) -> bool {
    lwjgl_from_winit(key).is_some_and(|code| code == binding_code)
}

pub fn binding_matches_mouse(binding_code: i32, button: MouseButton) -> bool {
    mouse_code(button).is_some_and(|code| code == binding_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vanilla_defaults_map_to_winit() {
        assert_eq!(lwjgl_from_winit(KeyCode::KeyW), Some(17));
        assert_eq!(lwjgl_from_winit(KeyCode::F5), Some(63));
        assert_eq!(lwjgl_from_winit(KeyCode::F11), Some(87));
        assert_eq!(mouse_code(MouseButton::Left), Some(-100));
        assert_eq!(mouse_code(MouseButton::Right), Some(-99));
    }
}
