#[cfg(not(feature = "wasm"))]
use crate::core::input::cache::InputCacheManager;
#[cfg(not(feature = "wasm"))]
use crate::core::input::events::ModifiersState;

#[cfg(not(feature = "wasm"))]
pub struct InputState {
    pub modifiers: ModifiersState,
    pub cache: InputCacheManager,
}

#[cfg(not(feature = "wasm"))]
impl InputState {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::default(),
            cache: InputCacheManager::new(),
        }
    }
}
