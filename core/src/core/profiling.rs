use serde::{Deserialize, Serialize};

use super::result::VulframResult;
use super::singleton::with_engine;

/// Profiling data structure for export
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingData {
    pub gamepad_processing_us: f64,
    pub event_loop_pump_us: f64,
    pub request_redraw_us: f64,
    pub serialization_us: f64,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
}

/// Get detailed profiling data from the last tick
pub fn vulfram_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        let data = ProfilingData {
            gamepad_processing_us: engine.profiling.gamepad_processing_ns as f64 / 1000.0,
            event_loop_pump_us: engine.profiling.event_loop_pump_ns as f64 / 1000.0,
            request_redraw_us: engine.profiling.request_redraw_ns as f64 / 1000.0,
            serialization_us: engine.profiling.serialization_ns as f64 / 1000.0,
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
