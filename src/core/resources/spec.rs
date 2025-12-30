use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct FrameComponent {
    pub time: f32,
    pub delta_time: f32,
    pub frame_index: u32,
    _padding: u32,
}

impl FrameComponent {
    pub fn new(time: f32, delta_time: f32, frame_index: u32) -> Self {
        Self {
            time,
            delta_time,
            frame_index,
            _padding: 0,
        }
    }
}
