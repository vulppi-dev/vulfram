use serde::{Deserialize, Serialize};

/// Represents the state of an input element (pressed or released)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementState {
    Released = 0,
    Pressed,
}

/// Represents the phase of a touch/gesture event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TouchPhase {
    Started = 0,
    Moved,
    Ended,
    Cancelled,
}

/// Represents keyboard modifier keys state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
