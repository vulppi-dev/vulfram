use std::thread;
use winit::event_loop::EventLoop;

use super::result::EngineResult;
use super::singleton::{ENGINE_INSTANCE, EngineCustomEvents, EngineSingleton, MAIN_THREAD_ID};
use super::state::EngineState;

/// Initialize the engine (must be called from the main thread)
pub fn engine_init() -> EngineResult {
    let _ = env_logger::try_init();
    let current_id = thread::current().id();

    if let Err(_) = MAIN_THREAD_ID.set(current_id) {
        if MAIN_THREAD_ID.get().unwrap() != &current_id {
            return EngineResult::WrongThread;
        }
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_some() {
            return EngineResult::AlreadyInitialized;
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
            return EngineResult::Success;
        }
    })
}

/// Dispose of the engine and clean up resources
pub fn engine_dispose() -> EngineResult {
    let current_id = thread::current().id();

    if let Some(main_id) = MAIN_THREAD_ID.get() {
        if &current_id != main_id {
            return EngineResult::WrongThread;
        }
    } else {
        return EngineResult::NotInitialized;
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        *opt = None;
    });

    EngineResult::Success
}
