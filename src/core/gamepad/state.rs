use crate::core::gamepad::cache::GamepadCacheManager;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::gilrs;

pub struct GamepadState {
    #[cfg(not(feature = "wasm"))]
    pub gilrs: Option<gilrs::Gilrs>,
    pub cache: GamepadCacheManager,
}

impl GamepadState {
    pub fn new() -> Self {
        #[cfg(not(feature = "wasm"))]
        let gilrs = match gilrs::Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => {
                log::warn!("Failed to initialize gamepad support: {:?}", e);
                None
            }
        };

        Self {
            #[cfg(not(feature = "wasm"))]
            gilrs,
            cache: GamepadCacheManager::new(),
        }
    }
}
