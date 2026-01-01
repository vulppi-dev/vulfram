use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec2, Vec4};
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
    pub view: Mat4,
    pub projection: Mat4,
    pub view_projection: Mat4,
    pub intensity_range: Vec2,
    pub spot_inner_outer: Vec2,
    pub kind_flags: UVec2, // x: kind, y: flags (bit 0: cast_shadow)
    pub _padding: UVec2,
}

impl LightComponent {
    pub const FLAG_CAST_SHADOW: u32 = 1 << 0;

    pub fn new(
        position: Vec4,
        direction: Vec4,
        color: Vec4,
        intensity: f32,
        range: f32,
        spot_inner_outer: Vec2,
        kind: LightKind,
        cast_shadow: bool,
    ) -> Self {
        let view = Mat4::IDENTITY;
        let projection = Mat4::IDENTITY;

        let mut flags = 0u32;
        if cast_shadow {
            flags |= Self::FLAG_CAST_SHADOW;
        }

        Self {
            position,
            direction,
            color,
            view,
            projection,
            view_projection: projection * view,
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
    pub cast_shadow: bool,
    pub is_dirty: bool,
}

impl LightRecord {
    pub fn new(data: LightComponent, layer_mask: u32, cast_shadow: bool) -> Self {
        Self {
            data,
            layer_mask,
            cast_shadow,
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
