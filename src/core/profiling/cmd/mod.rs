use serde::{Deserialize, Serialize};

use crate::core::VulframResult;
use crate::core::singleton::with_engine;

/// Profiling data structure for export
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingData {
    pub command_processing_us: f64,
    pub gamepad_processing_us: f64,
    pub event_loop_pump_us: f64,
    pub request_redraw_us: f64,
    pub serialization_us: f64,
    pub render_total_us: f64,
    pub render_shadow_us: f64,
    pub render_windows_us: f64,
    pub frame_delta_us: f64,
    pub fps_instant: f64,
    pub gpu_supported: bool,
    pub gpu_shadow_us: f64,
    pub gpu_light_cull_us: f64,
    pub gpu_forward_us: f64,
    pub gpu_compose_us: f64,
    pub gpu_total_us: f64,
    pub window_fps: Vec<WindowFps>,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowFps {
    pub window_id: u32,
    pub fps_instant: f64,
    pub frame_delta_us: f64,
}

/// Get detailed profiling data from the last tick
pub fn vulfram_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        let mut window_fps = Vec::with_capacity(engine.window.states.len());
        for (&window_id, window_state) in &engine.window.states {
            window_fps.push(WindowFps {
                window_id,
                fps_instant: window_state.fps_instant,
                frame_delta_us: window_state.last_frame_delta_ns as f64 / 1000.0,
            });
        }
        let data = ProfilingData {
            command_processing_us: engine.profiling.command_processing_ns as f64 / 1000.0,
            gamepad_processing_us: engine.profiling.gamepad_processing_ns as f64 / 1000.0,
            event_loop_pump_us: engine.profiling.event_loop_pump_ns as f64 / 1000.0,
            request_redraw_us: engine.profiling.request_redraw_ns as f64 / 1000.0,
            serialization_us: engine.profiling.serialization_ns as f64 / 1000.0,
            render_total_us: engine.profiling.render_total_ns as f64 / 1000.0,
            render_shadow_us: engine.profiling.render_shadow_ns as f64 / 1000.0,
            render_windows_us: engine.profiling.render_windows_ns as f64 / 1000.0,
            frame_delta_us: engine.profiling.frame_delta_ns as f64 / 1000.0,
            fps_instant: if engine.profiling.frame_delta_ns > 0 {
                1_000_000_000.0 / engine.profiling.frame_delta_ns as f64
            } else {
                0.0
            },
            gpu_supported: engine.gpu_profiler.is_some(),
            gpu_shadow_us: engine.profiling.gpu_shadow_ns as f64 / 1000.0,
            gpu_light_cull_us: engine.profiling.gpu_light_cull_ns as f64 / 1000.0,
            gpu_forward_us: engine.profiling.gpu_forward_ns as f64 / 1000.0,
            gpu_compose_us: engine.profiling.gpu_compose_ns as f64 / 1000.0,
            gpu_total_us: engine.profiling.gpu_total_ns as f64 / 1000.0,
            window_fps,
            total_events_dispatched: engine.profiling.total_events_dispatched,
            total_events_cached: engine.profiling.total_events_cached,
        };

        // Serialize profiling data
        let serialized_data = match rmp_serde::to_vec_named(&data) {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };

        let data_length = serialized_data.len();

        // Transfer ownership via Box::into_raw (zero-copy)
        let boxed = serialized_data.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u8;

        unsafe {
            *out_ptr = ptr;
            *out_length = data_length;
        }

        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
