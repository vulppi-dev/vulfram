use std::thread;
use winit::event_loop::EventLoop;

use super::VulframResult;
use super::singleton::{ENGINE_INSTANCE, EngineCustomEvents, EngineSingleton, MAIN_THREAD_ID};
use super::state::EngineState;

/// Initialize the engine (must be called from the main thread)
pub fn vulfram_init() -> VulframResult {
    let _ = env_logger::try_init();
    let current_id = thread::current().id();

    if let Err(_) = MAIN_THREAD_ID.set(current_id) {
        if MAIN_THREAD_ID.get().unwrap() != &current_id {
            return VulframResult::WrongThread;
        }
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_some() {
            return VulframResult::AlreadyInitialized;
        } else {
            let event_loop = EventLoop::<EngineCustomEvents>::with_user_event()
                .build()
                .unwrap();
            let proxy = event_loop.create_proxy();

            *opt = Some(EngineSingleton {
                state: EngineState::new(),
                event_loop: Some(event_loop),
                proxy: Some(proxy),
            });
            return VulframResult::Success;
        }
    })
}

/// Dispose of the engine and clean up resources
pub fn vulfram_dispose() -> VulframResult {
    let current_id = thread::current().id();

    if let Some(main_id) = MAIN_THREAD_ID.get() {
        if &current_id != main_id {
            return VulframResult::WrongThread;
        }
    } else {
        return VulframResult::NotInitialized;
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();

        // Explicitly clean up all render states before dropping
        if let Some(ref mut singleton) = *opt {
            for (_window_id, window_state) in singleton.state.window.states.iter_mut() {
                window_state.render_state.drop_all();
            }
            // Clear all windows to drop GPU resources
            singleton.state.window.states.clear();
            singleton.state.window.window_id_map.clear();
        }

        *opt = None;
    });

    VulframResult::Success
}
