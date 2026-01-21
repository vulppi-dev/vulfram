use crate::core::cmd::engine_process_batch;
use crate::core::platforms::PlatformProxy;

use super::VulframResult;
use super::singleton::with_engine_singleton;

/// Main engine tick - processes events and updates state
pub fn vulfram_tick(time: u64, delta_time: u32) -> VulframResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;
        engine.state.event_queue.clear();

        if !engine.state.cmd_queue.is_empty() {
            let batch = std::mem::take(&mut engine.state.cmd_queue);
            let result = engine_process_batch(&mut engine.state, &mut engine.platform, batch);
            if result != VulframResult::Success {
                return result;
            }
        }

        // Reset profiling counters
        engine.state.profiling.gamepad_processing_ns = 0;
        engine.state.profiling.event_loop_pump_ns = 0;
        engine.state.profiling.total_events_cached = 0;
        engine.state.profiling.custom_events_ns = 0;

        let events_before = engine.state.event_queue.len();

        // MARK: Gamepad Processing
        engine.state.profiling.gamepad_processing_ns =
            engine.platform.process_gamepads(&mut engine.state);

        // MARK: Event Loop Pump
        engine.state.profiling.event_loop_pump_ns = engine.platform.pump_events(&mut engine.state);

        let events_after = engine.state.event_queue.len();
        engine.state.profiling.total_events_dispatched = events_after - events_before;

        // MARK: Render Frame Lifecycle
        engine.state.frame_index = engine.state.frame_index.wrapping_add(1);
        let frame_index = engine.state.frame_index;
        for window_state in engine.state.window.states.values_mut() {
            window_state.render_state.begin_frame(frame_index);
        }

        // MARK: Request Redraw
        engine.state.profiling.request_redraw_ns = engine.platform.render(&mut engine.state);
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
