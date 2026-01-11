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

pub const MATERIAL_FALLBACK_ID: u32 = 0;
pub const STANDARD_INPUTS_PER_MATERIAL: u32 = 8;
pub const STANDARD_TEXTURE_SLOTS: usize = 8;
pub const STANDARD_INVALID_SLOT: u32 = u32::MAX;
pub const PBR_INPUTS_PER_MATERIAL: u32 = 8;
pub const PBR_TEXTURE_SLOTS: usize = 8;
pub const PBR_INVALID_SLOT: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialStandardParams {
    pub input_indices: glam::UVec4,
    pub inputs_offset_count: glam::UVec2,
    pub surface_flags: glam::UVec2,
    pub texture_slots: [glam::UVec4; 2],
    pub sampler_indices: [glam::UVec4; 2],
    pub tex_sources: [glam::UVec4; 2],
    pub atlas_layers: [glam::UVec4; 2],
    pub atlas_scale_bias: [glam::Vec4; STANDARD_TEXTURE_SLOTS],
}

impl Default for MaterialStandardParams {
    fn default() -> Self {
        Self {
            input_indices: glam::UVec4::new(0, 1, 2, 3),
            inputs_offset_count: glam::UVec2::new(0, STANDARD_INPUTS_PER_MATERIAL),
            surface_flags: glam::UVec2::new(SurfaceType::Opaque as u32, 0),
            texture_slots: [glam::UVec4::splat(STANDARD_INVALID_SLOT); 2],
            sampler_indices: [glam::UVec4::ZERO; 2],
            tex_sources: [glam::UVec4::splat(2); 2],
            atlas_layers: [glam::UVec4::ZERO; 2],
            atlas_scale_bias: [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); STANDARD_TEXTURE_SLOTS],
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialPbrParams {
    pub input_indices: glam::UVec4,
    pub inputs_offset_count: glam::UVec2,
    pub surface_flags: glam::UVec2,
    pub texture_slots: [glam::UVec4; 2],
    pub sampler_indices: [glam::UVec4; 2],
    pub tex_sources: [glam::UVec4; 2],
    pub atlas_layers: [glam::UVec4; 2],
    pub atlas_scale_bias: [glam::Vec4; PBR_TEXTURE_SLOTS],
}

impl Default for MaterialPbrParams {
    fn default() -> Self {
        Self {
            input_indices: glam::UVec4::new(0, 1, 2, 3),
            inputs_offset_count: glam::UVec2::new(0, PBR_INPUTS_PER_MATERIAL),
            surface_flags: glam::UVec2::new(SurfaceType::Opaque as u32, 0),
            texture_slots: [glam::UVec4::splat(PBR_INVALID_SLOT); 2],
            sampler_indices: [glam::UVec4::ZERO; 2],
            tex_sources: [glam::UVec4::splat(2); 2],
            atlas_layers: [glam::UVec4::ZERO; 2],
            atlas_scale_bias: [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); PBR_TEXTURE_SLOTS],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialStandardRecord {
    pub label: Option<String>,
    pub data: MaterialStandardParams,
    pub inputs: Vec<Vec4>,
    pub texture_ids: [u32; STANDARD_TEXTURE_SLOTS],
    pub surface_type: SurfaceType,
    pub is_dirty: bool,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl MaterialStandardRecord {
    pub fn new(label: Option<String>, data: MaterialStandardParams) -> Self {
        let mut inputs = vec![Vec4::ZERO; STANDARD_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ONE;
        inputs[2] = Vec4::new(32.0, 0.0, 0.0, 0.0);
        Self {
            label,
            data,
            inputs,
            texture_ids: [STANDARD_INVALID_SLOT; STANDARD_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[derive(Debug, Clone)]
pub struct MaterialPbrRecord {
    pub label: Option<String>,
    pub data: MaterialPbrParams,
    pub inputs: Vec<Vec4>,
    pub texture_ids: [u32; PBR_TEXTURE_SLOTS],
    pub surface_type: SurfaceType,
    pub is_dirty: bool,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl MaterialPbrRecord {
    pub fn new(label: Option<String>, data: MaterialPbrParams) -> Self {
        let mut inputs = vec![Vec4::ZERO; PBR_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ZERO;
        inputs[2] = Vec4::new(0.0, 1.0, 1.0, 0.0);
        inputs[3] = Vec4::new(1.0, 0.0, 0.0, 0.0);
        Self {
            label,
            data,
            inputs,
            texture_ids: [PBR_INVALID_SLOT; PBR_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}
