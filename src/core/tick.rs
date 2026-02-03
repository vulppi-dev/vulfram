use crate::core::audio::{process_audio_listener_binding, process_audio_source_bindings};
use crate::core::cmd::engine_process_batch;
use crate::core::platforms::PlatformProxy;

#[cfg(feature = "wasm")]
use js_sys::Date;
#[cfg(not(feature = "wasm"))]
use std::time::Instant;

use super::VulframResult;
use super::singleton::with_engine_singleton;

/// Main engine tick - processes events and updates state
pub fn vulfram_tick(time: u64, delta_time: u32) -> VulframResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;
        engine.state.event_queue.clear();

        // Reset profiling counters
        engine.state.profiling.command_processing_ns = 0;
        engine.state.profiling.gamepad_processing_ns = 0;
        engine.state.profiling.event_loop_pump_ns = 0;
        engine.state.profiling.total_events_cached = 0;
        engine.state.profiling.custom_events_ns = 0;
        engine.state.profiling.render_total_ns = 0;
        engine.state.profiling.render_shadow_ns = 0;
        engine.state.profiling.render_windows_ns = 0;
        engine.state.profiling.gpu_shadow_ns = 0;
        engine.state.profiling.gpu_light_cull_ns = 0;
        engine.state.profiling.gpu_forward_ns = 0;
        engine.state.profiling.gpu_compose_ns = 0;
        engine.state.profiling.gpu_total_ns = 0;
        engine.state.profiling.frame_delta_ns = (delta_time as u64).saturating_mul(1_000_000);

        if !engine.state.cmd_queue.is_empty() {
            // MARK: Command Processing
            #[cfg(not(feature = "wasm"))]
            let cmd_start = Instant::now();
            #[cfg(feature = "wasm")]
            let cmd_start = (Date::now() * 1_000_000.0) as u64;
            let batch = std::mem::take(&mut engine.state.cmd_queue);
            let result = engine_process_batch(&mut engine.state, &mut engine.platform, batch);
            #[cfg(not(feature = "wasm"))]
            {
                engine.state.profiling.command_processing_ns =
                    cmd_start.elapsed().as_nanos() as u64;
            }
            #[cfg(feature = "wasm")]
            {
                let now = (Date::now() * 1_000_000.0) as u64;
                engine.state.profiling.command_processing_ns = now.saturating_sub(cmd_start);
            }
            if result != VulframResult::Success {
                return result;
            }
        }

        process_audio_listener_binding(&mut engine.state);
        process_audio_source_bindings(&mut engine.state);
        crate::core::resources::process_async_texture_results(&mut engine.state);
        let audio_events = engine.state.audio.drain_events();
        for event in audio_events {
            engine
                .state
                .event_queue
                .push(crate::core::cmd::EngineEvent::System(
                    crate::core::system::events::SystemEvent::AudioReady {
                        resource_id: event.resource_id,
                        success: event.success,
                        message: event.message,
                    },
                ));
        }

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
