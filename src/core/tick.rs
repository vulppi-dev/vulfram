use std::time::Instant;
use winit::platform::pump_events::EventLoopExtPumpEvents;

use super::VulframResult;
use super::gamepad::process_gilrs_event;
use super::singleton::with_engine_singleton;

/// Main engine tick - processes events and updates state
pub fn vulfram_tick(time: u64, delta_time: u32) -> VulframResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;
        engine.state.event_queue.clear();

        // Reset profiling counters
        engine.state.profiling.gamepad_processing_ns = 0;
        engine.state.profiling.event_loop_pump_ns = 0;
        engine.state.profiling.total_events_cached = 0;
        engine.state.profiling.custom_events_ns = 0;

        let events_before = engine.state.event_queue.len();

        // MARK: Gamepad Processing
        let gamepad_start = Instant::now();
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut engine.state.gamepad.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            process_gilrs_event(&mut engine.state, event);
        }
        engine.state.profiling.gamepad_processing_ns = gamepad_start.elapsed().as_nanos() as u64;

        // MARK: Event Loop Pump
        if let Some(event_loop) = &mut engine.event_loop {
            // Only set control flow if it's not already set to Poll
            // This avoids unnecessary overhead
            let pump_start = Instant::now();

            // pump_app_events with timeout=None processes all pending events
            // without blocking or yielding to the OS
            event_loop.pump_app_events(None, &mut engine.state);

            let total_pump_time = pump_start.elapsed().as_nanos() as u64;
            // Subtract custom events time (window creation) from event loop pump time
            engine.state.profiling.event_loop_pump_ns =
                total_pump_time.saturating_sub(engine.state.profiling.custom_events_ns);
        }

        let events_after = engine.state.event_queue.len();
        engine.state.profiling.total_events_dispatched = events_after - events_before;

        // MARK: Request Redraw
        let start = std::time::Instant::now();

        for window_state in engine.state.window.states.values_mut() {
            window_state.window.request_redraw();
        }

        engine.state.profiling.request_redraw_ns = start.elapsed().as_nanos() as u64;
    }) {
        Err(e) => e,
        Ok(_) => VulframResult::Success,
    }
}
