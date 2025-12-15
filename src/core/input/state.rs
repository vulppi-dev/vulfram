use crate::core::input::cache::InputCacheManager;
use crate::core::input::events::ModifiersState;

pub struct InputState {
    pub modifiers: ModifiersState,
    pub cache: InputCacheManager,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::default(),
            cache: InputCacheManager::new(),
        }
    }
}
