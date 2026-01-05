use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use serde::{Deserialize, Serialize};

pub const MATERIAL_FALLBACK_ID: u32 = 0;

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct MaterialUnlitComponent {
    pub base_color: Vec4,
}

impl Default for MaterialUnlitComponent {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialUnlitRecord {
    pub data: MaterialUnlitComponent,
    pub is_dirty: bool,
}

impl MaterialUnlitRecord {
    pub fn new(data: MaterialUnlitComponent) -> Self {
        Self {
            data,
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
pub struct MaterialLambertComponent {
    pub base_color: Vec4,
}

impl Default for MaterialLambertComponent {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialLambertRecord {
    pub data: MaterialLambertComponent,
    pub is_dirty: bool,
}

impl MaterialLambertRecord {
    pub fn new(data: MaterialLambertComponent) -> Self {
        Self {
            data,
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
