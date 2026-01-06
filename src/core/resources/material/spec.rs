use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(u32)]
pub enum SurfaceType {
    Opaque = 0,
    Masked = 1,
    Transparent = 2,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(u32)]
pub enum ShadingModel {
    Standard = 0,
    Pbr = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(u32)]
pub enum MaterialExec {
    Direct = 0,
    Graph = 1,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialKey {
    pub surface: SurfaceType,
    pub shading: ShadingModel,
    pub exec: MaterialExec,
    pub flags: u32,
}

pub const MATERIAL_FALLBACK_ID: u32 = 0;
pub const STANDARD_INPUTS_PER_MATERIAL: u32 = 8;
pub const STANDARD_TEXTURE_SLOTS: usize = 8;
pub const STANDARD_INVALID_SLOT: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialStandardParams {
    pub input_indices: glam::UVec4,
    pub inputs_offset_count: glam::UVec2,
    pub surface_flags: glam::UVec2,
    pub texture_slots: [glam::UVec4; 2],
    pub sampler_indices: [glam::UVec4; 2],
}

impl Default for MaterialStandardParams {
    fn default() -> Self {
        Self {
            input_indices: glam::UVec4::new(0, 1, 2, 3),
            inputs_offset_count: glam::UVec2::new(0, STANDARD_INPUTS_PER_MATERIAL),
            surface_flags: glam::UVec2::new(SurfaceType::Opaque as u32, 0),
            texture_slots: [glam::UVec4::splat(STANDARD_INVALID_SLOT); 2],
            sampler_indices: [glam::UVec4::ZERO; 2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialStandardRecord {
    pub data: MaterialStandardParams,
    pub inputs: Vec<Vec4>,
    pub surface_type: SurfaceType,
    pub is_dirty: bool,
}

impl MaterialStandardRecord {
    pub fn new(data: MaterialStandardParams) -> Self {
        let mut inputs = vec![Vec4::ZERO; STANDARD_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ONE;
        inputs[2] = Vec4::new(32.0, 0.0, 0.0, 0.0);
        Self {
            data,
            inputs,
            surface_type: SurfaceType::Opaque,
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
