use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct FrameSpec {
    pub time: f32,
    pub delta_time: f32,
    pub frame_index: u32,
    _padding: u32,
}
