use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ComponentContainer<T> {
    pub data: T,
    pub layer_mask: u32,
    pub is_dirty: bool,
}

impl<T> ComponentContainer<T> {
    pub fn new(data: T, layer_mask: u32) -> Self {
        Self {
            data,
            layer_mask,
            is_dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct FrameSpec {
    pub time: f32,
    pub delta_time: f32,
    pub frame_index: u32,
    _padding: u32,
}
