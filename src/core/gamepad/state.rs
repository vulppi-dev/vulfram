use crate::core::gamepad::cache::GamepadCacheManager;

pub struct GamepadState {
    pub gilrs: Option<gilrs::Gilrs>,
    pub cache: GamepadCacheManager,
}

impl GamepadState {
    pub fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => {
                log::warn!("Failed to initialize gamepad support: {:?}", e);
                None
            }
        };

        Self {
            gilrs,
            cache: GamepadCacheManager::new(),
        }
    }
}
