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

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialStandardComponent {
    pub base_color: Vec4,
}

impl Default for MaterialStandardComponent {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialStandardRecord {
    pub data: MaterialStandardComponent,
    pub surface_type: SurfaceType,
    pub is_dirty: bool,
}

impl MaterialStandardRecord {
    pub fn new(data: MaterialStandardComponent) -> Self {
        Self {
            data,
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
