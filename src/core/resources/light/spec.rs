use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec2, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightKind {
    Directional = 0,
    Point,
    Spot,
    Ambient,
    Hemisphere,
}

impl LightKind {
    pub fn to_u32(self) -> u32 {
        match self {
            LightKind::Directional => 0,
            LightKind::Point => 1,
            LightKind::Spot => 2,
            LightKind::Ambient => 3,
            LightKind::Hemisphere => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct LightComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub color: Vec4,
    pub ground_color: Vec4,
    pub view: Mat4,
    pub projection: Mat4,
    pub view_projection: Mat4,
    pub intensity_range: Vec2,
    pub spot_inner_outer: Vec2,
    pub kind_flags: UVec2, // x: kind, y: flags (bit 0: cast_shadow)
    pub shadow_index: u32,
    pub _padding: u32,
}

impl LightComponent {
    pub const FLAG_CAST_SHADOW: u32 = 1 << 0;

    pub fn new(
        position: Vec4,
        direction: Vec4,
        color: Vec4,
        ground_color: Vec4,
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
            ground_color,
            view,
            projection,
            view_projection: projection * view,
            intensity_range: Vec2::new(intensity, range),
            spot_inner_outer,
            kind_flags: UVec2::new(kind.to_u32(), flags),
            shadow_index: 0xFFFFFFFF,
            _padding: 0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct LightRecord {
    pub label: Option<String>,
    pub data: LightComponent,
    pub layer_mask: u32,
    pub cast_shadow: bool,
    pub is_dirty: bool,
}

impl LightRecord {
    pub fn new(
        label: Option<String>,
        data: LightComponent,
        layer_mask: u32,
        cast_shadow: bool,
    ) -> Self {
        Self {
            label,
            data,
            layer_mask,
            cast_shadow,
            is_dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
}
