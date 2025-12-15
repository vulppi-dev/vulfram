#[derive(Debug, Clone, Default)]
pub struct TickProfiling {
    pub gamepad_processing_ns: u64,
    pub event_loop_pump_ns: u64,
    pub request_redraw_ns: u64,
    pub serialization_ns: u64,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
    pub custom_events_ns: u64,
}
