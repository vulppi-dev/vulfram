/// Convert gilrs Button to u32
pub fn convert_gilrs_button(button: gilrs::Button) -> u32 {
    use gilrs::Button;

    match button {
        Button::South => 0,
        Button::East => 1,
        Button::West => 2,
        Button::North => 3,
        Button::LeftTrigger => 4,
        Button::RightTrigger => 5,
        Button::LeftTrigger2 => 6,
        Button::RightTrigger2 => 7,
        Button::Select => 8,
        Button::Start => 9,
        Button::Mode => 10,
        Button::LeftThumb => 11,
        Button::RightThumb => 12,
        Button::DPadUp => 13,
        Button::DPadDown => 14,
        Button::DPadLeft => 15,
        Button::DPadRight => 16,
        _ => 255,
    }
}

/// Convert gilrs Axis to u32
pub fn convert_gilrs_axis(axis: gilrs::Axis) -> u32 {
    use gilrs::Axis;

    match axis {
        Axis::LeftStickX => 0,
        Axis::LeftStickY => 1,
        Axis::RightStickX => 2,
        Axis::RightStickY => 3,
        Axis::LeftZ => 4,
        Axis::RightZ => 5,
        _ => 255,
    }
}
