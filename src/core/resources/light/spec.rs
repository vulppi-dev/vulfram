use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2, Vec4};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum LightKind {
    Directional = 0,
    Point,
    Spot,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct LightComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub color: Vec4,
    pub intensity_range: Vec2,
    pub spot_inner_outer: Vec2,
    pub kind_flags: UVec2,
    pub _padding: UVec2,
}

impl LightComponent {
    pub fn new(
        position: Vec4,
        direction: Vec4,
        color: Vec4,
        intensity: f32,
        range: f32,
        spot_inner_outer: Vec2,
        kind: LightKind,
        flags: u32,
    ) -> Self {
        Self {
            position,
            direction,
            color,
            intensity_range: Vec2::new(intensity, range),
            spot_inner_outer,
            kind_flags: UVec2::new(kind as u32, flags),
            _padding: UVec2::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightRecord {
    pub data: LightComponent,
    pub layer_mask: u32,
    pub is_dirty: bool,
}

impl LightRecord {
    pub fn new(data: LightComponent, layer_mask: u32) -> Self {
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
