mod common;
#[cfg(not(feature = "wasm"))]
mod converters;
mod keyboard;
mod pointer;

pub use common::ElementState;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
pub use common::ModifiersState;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use common::TouchPhase;
#[cfg(not(feature = "wasm"))]
pub use converters::{
    convert_key_code, convert_key_location, convert_mouse_button, convert_touch_phase,
};
pub use keyboard::KeyboardEvent;
pub use pointer::PointerEvent;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
pub use pointer::ScrollDelta;
