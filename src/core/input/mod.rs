#[cfg(not(feature = "wasm"))]
pub mod cache;
pub mod events;
#[cfg(not(feature = "wasm"))]
pub mod state;

#[cfg(not(feature = "wasm"))]
pub use cache::InputCacheManager;
#[cfg(not(feature = "wasm"))]
pub use events::{ElementState, KeyboardEvent, ModifiersState, PointerEvent, ScrollDelta};
#[cfg(not(feature = "wasm"))]
pub use events::{
    convert_key_code, convert_key_location, convert_mouse_button, convert_touch_phase,
};
#[cfg(not(feature = "wasm"))]
pub use state::InputState;
