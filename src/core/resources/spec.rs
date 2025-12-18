use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec2, Vec4};
use serde::{Deserialize, Serialize};

// MARK: - Component Container

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

// MARK: - Frame Spec

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct FrameSpec {
    pub time: f32,
    pub delta_time: f32,
    pub frame_index: u32,
    _padding: u32,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct CameraComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub up: Vec4,
    pub near_far: Vec2,
    pub kind_flags: UVec2,

    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
}
