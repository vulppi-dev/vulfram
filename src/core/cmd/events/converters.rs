use super::common::TouchPhase;
use super::gamepad::{GamepadAxis, GamepadButton};
use super::keyboard::{KeyCode, KeyLocation};
use super::pointer::MouseButton;

/// Convert winit TouchPhase to our TouchPhase
pub fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

/// Convert winit MouseButton to our MouseButton
pub fn convert_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(id) => MouseButton::Other(id as u8),
    }
}

/// Convert winit KeyLocation to our KeyLocation
pub fn convert_key_location(location: winit::keyboard::KeyLocation) -> KeyLocation {
    match location {
        winit::keyboard::KeyLocation::Standard => KeyLocation::Standard,
        winit::keyboard::KeyLocation::Left => KeyLocation::Left,
        winit::keyboard::KeyLocation::Right => KeyLocation::Right,
        winit::keyboard::KeyLocation::Numpad => KeyLocation::Numpad,
    }
}

/// Convert gilrs Button to our GamepadButton
pub fn convert_gilrs_button(button: gilrs::Button) -> GamepadButton {
    use gilrs::Button;

    match button {
        Button::South => GamepadButton::South,
        Button::East => GamepadButton::East,
        Button::West => GamepadButton::West,
        Button::North => GamepadButton::North,
        Button::LeftTrigger => GamepadButton::LeftBumper,
        Button::RightTrigger => GamepadButton::RightBumper,
        Button::LeftTrigger2 => GamepadButton::LeftTrigger,
        Button::RightTrigger2 => GamepadButton::RightTrigger,
        Button::Select => GamepadButton::Select,
        Button::Start => GamepadButton::Start,
        Button::Mode => GamepadButton::Mode,
        Button::LeftThumb => GamepadButton::LeftStick,
        Button::RightThumb => GamepadButton::RightStick,
        Button::DPadUp => GamepadButton::DpadUp,
        Button::DPadDown => GamepadButton::DpadDown,
        Button::DPadLeft => GamepadButton::DpadLeft,
        Button::DPadRight => GamepadButton::DpadRight,
        _ => GamepadButton::Other(255),
    }
}

/// Convert gilrs Axis to our GamepadAxis
pub fn convert_gilrs_axis(axis: gilrs::Axis) -> GamepadAxis {
    use gilrs::Axis;

    match axis {
        Axis::LeftStickX => GamepadAxis::LeftStickX,
        Axis::LeftStickY => GamepadAxis::LeftStickY,
        Axis::RightStickX => GamepadAxis::RightStickX,
        Axis::RightStickY => GamepadAxis::RightStickY,
        Axis::LeftZ => GamepadAxis::LeftTrigger,
        Axis::RightZ => GamepadAxis::RightTrigger,
        _ => GamepadAxis::Other(255),
    }
}

/// Convert winit PhysicalKey to our KeyCode
pub fn convert_key_code(physical_key: &winit::keyboard::PhysicalKey) -> KeyCode {
    use winit::keyboard::KeyCode as WKeyCode;
    use winit::keyboard::PhysicalKey;

    match physical_key {
        PhysicalKey::Code(code) => match code {
            // Writing System Keys
            WKeyCode::Backquote => KeyCode::Backquote,
            WKeyCode::Backslash => KeyCode::Backslash,
            WKeyCode::BracketLeft => KeyCode::BracketLeft,
            WKeyCode::BracketRight => KeyCode::BracketRight,
            WKeyCode::Comma => KeyCode::Comma,
            WKeyCode::Digit0 => KeyCode::Digit0,
            WKeyCode::Digit1 => KeyCode::Digit1,
            WKeyCode::Digit2 => KeyCode::Digit2,
            WKeyCode::Digit3 => KeyCode::Digit3,
            WKeyCode::Digit4 => KeyCode::Digit4,
            WKeyCode::Digit5 => KeyCode::Digit5,
            WKeyCode::Digit6 => KeyCode::Digit6,
            WKeyCode::Digit7 => KeyCode::Digit7,
            WKeyCode::Digit8 => KeyCode::Digit8,
            WKeyCode::Digit9 => KeyCode::Digit9,
            WKeyCode::Equal => KeyCode::Equal,
            WKeyCode::IntlBackslash => KeyCode::IntlBackslash,
            WKeyCode::IntlRo => KeyCode::IntlRo,
            WKeyCode::IntlYen => KeyCode::IntlYen,
            WKeyCode::KeyA => KeyCode::KeyA,
            WKeyCode::KeyB => KeyCode::KeyB,
            WKeyCode::KeyC => KeyCode::KeyC,
            WKeyCode::KeyD => KeyCode::KeyD,
            WKeyCode::KeyE => KeyCode::KeyE,
            WKeyCode::KeyF => KeyCode::KeyF,
            WKeyCode::KeyG => KeyCode::KeyG,
            WKeyCode::KeyH => KeyCode::KeyH,
            WKeyCode::KeyI => KeyCode::KeyI,
            WKeyCode::KeyJ => KeyCode::KeyJ,
            WKeyCode::KeyK => KeyCode::KeyK,
            WKeyCode::KeyL => KeyCode::KeyL,
            WKeyCode::KeyM => KeyCode::KeyM,
            WKeyCode::KeyN => KeyCode::KeyN,
            WKeyCode::KeyO => KeyCode::KeyO,
            WKeyCode::KeyP => KeyCode::KeyP,
            WKeyCode::KeyQ => KeyCode::KeyQ,
            WKeyCode::KeyR => KeyCode::KeyR,
            WKeyCode::KeyS => KeyCode::KeyS,
            WKeyCode::KeyT => KeyCode::KeyT,
            WKeyCode::KeyU => KeyCode::KeyU,
            WKeyCode::KeyV => KeyCode::KeyV,
            WKeyCode::KeyW => KeyCode::KeyW,
            WKeyCode::KeyX => KeyCode::KeyX,
            WKeyCode::KeyY => KeyCode::KeyY,
            WKeyCode::KeyZ => KeyCode::KeyZ,
            WKeyCode::Minus => KeyCode::Minus,
            WKeyCode::Period => KeyCode::Period,
            WKeyCode::Quote => KeyCode::Quote,
            WKeyCode::Semicolon => KeyCode::Semicolon,
            WKeyCode::Slash => KeyCode::Slash,

            // Functional Keys
            WKeyCode::AltLeft => KeyCode::AltLeft,
            WKeyCode::AltRight => KeyCode::AltRight,
            WKeyCode::Backspace => KeyCode::Backspace,
            WKeyCode::CapsLock => KeyCode::CapsLock,
            WKeyCode::ContextMenu => KeyCode::ContextMenu,
            WKeyCode::ControlLeft => KeyCode::ControlLeft,
            WKeyCode::ControlRight => KeyCode::ControlRight,
            WKeyCode::Enter => KeyCode::Enter,
            WKeyCode::SuperLeft => KeyCode::SuperLeft,
            WKeyCode::SuperRight => KeyCode::SuperRight,
            WKeyCode::ShiftLeft => KeyCode::ShiftLeft,
            WKeyCode::ShiftRight => KeyCode::ShiftRight,
            WKeyCode::Space => KeyCode::Space,
            WKeyCode::Tab => KeyCode::Tab,

            // Control Keys
            WKeyCode::Delete => KeyCode::Delete,
            WKeyCode::End => KeyCode::End,
            WKeyCode::Help => KeyCode::Help,
            WKeyCode::Home => KeyCode::Home,
            WKeyCode::Insert => KeyCode::Insert,
            WKeyCode::PageDown => KeyCode::PageDown,
            WKeyCode::PageUp => KeyCode::PageUp,

            // Arrow Keys
            WKeyCode::ArrowDown => KeyCode::ArrowDown,
            WKeyCode::ArrowLeft => KeyCode::ArrowLeft,
            WKeyCode::ArrowRight => KeyCode::ArrowRight,
            WKeyCode::ArrowUp => KeyCode::ArrowUp,

            // Numpad Keys
            WKeyCode::NumLock => KeyCode::NumLock,
            WKeyCode::Numpad0 => KeyCode::Numpad0,
            WKeyCode::Numpad1 => KeyCode::Numpad1,
            WKeyCode::Numpad2 => KeyCode::Numpad2,
            WKeyCode::Numpad3 => KeyCode::Numpad3,
            WKeyCode::Numpad4 => KeyCode::Numpad4,
            WKeyCode::Numpad5 => KeyCode::Numpad5,
            WKeyCode::Numpad6 => KeyCode::Numpad6,
            WKeyCode::Numpad7 => KeyCode::Numpad7,
            WKeyCode::Numpad8 => KeyCode::Numpad8,
            WKeyCode::Numpad9 => KeyCode::Numpad9,
            WKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            WKeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
            WKeyCode::NumpadClear => KeyCode::NumpadClear,
            WKeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
            WKeyCode::NumpadComma => KeyCode::NumpadComma,
            WKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            WKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            WKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            WKeyCode::NumpadEqual => KeyCode::NumpadEqual,
            WKeyCode::NumpadHash => KeyCode::NumpadHash,
            WKeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
            WKeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
            WKeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
            WKeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
            WKeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
            WKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            WKeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
            WKeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
            WKeyCode::NumpadStar => KeyCode::NumpadStar,
            WKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,

            // Function Keys
            WKeyCode::Escape => KeyCode::Escape,
            WKeyCode::F1 => KeyCode::F1,
            WKeyCode::F2 => KeyCode::F2,
            WKeyCode::F3 => KeyCode::F3,
            WKeyCode::F4 => KeyCode::F4,
            WKeyCode::F5 => KeyCode::F5,
            WKeyCode::F6 => KeyCode::F6,
            WKeyCode::F7 => KeyCode::F7,
            WKeyCode::F8 => KeyCode::F8,
            WKeyCode::F9 => KeyCode::F9,
            WKeyCode::F10 => KeyCode::F10,
            WKeyCode::F11 => KeyCode::F11,
            WKeyCode::F12 => KeyCode::F12,
            WKeyCode::F13 => KeyCode::F13,
            WKeyCode::F14 => KeyCode::F14,
            WKeyCode::F15 => KeyCode::F15,
            WKeyCode::F16 => KeyCode::F16,
            WKeyCode::F17 => KeyCode::F17,
            WKeyCode::F18 => KeyCode::F18,
            WKeyCode::F19 => KeyCode::F19,
            WKeyCode::F20 => KeyCode::F20,
            WKeyCode::F21 => KeyCode::F21,
            WKeyCode::F22 => KeyCode::F22,
            WKeyCode::F23 => KeyCode::F23,
            WKeyCode::F24 => KeyCode::F24,

            // Lock Keys
            WKeyCode::ScrollLock => KeyCode::ScrollLock,

            // Media Keys
            WKeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
            WKeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
            WKeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
            WKeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
            WKeyCode::MediaStop => KeyCode::MediaStop,
            WKeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
            WKeyCode::MediaTrackPrevious => KeyCode::MediaTrackPrevious,

            // Browser Keys
            WKeyCode::BrowserBack => KeyCode::BrowserBack,
            WKeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
            WKeyCode::BrowserForward => KeyCode::BrowserForward,
            WKeyCode::BrowserHome => KeyCode::BrowserHome,
            WKeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
            WKeyCode::BrowserSearch => KeyCode::BrowserSearch,
            WKeyCode::BrowserStop => KeyCode::BrowserStop,

            // System Keys
            WKeyCode::PrintScreen => KeyCode::PrintScreen,
            WKeyCode::Pause => KeyCode::Pause,

            _ => KeyCode::Unidentified,
        },
        PhysicalKey::Unidentified(_) => KeyCode::Unidentified,
    }
}
