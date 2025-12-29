use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct ModelComponent {
    pub transform: Mat4,
    pub translation: Vec4,
    pub rotation: Vec4,
    pub scale: Vec4,
}

impl ModelComponent {
    pub fn new(transform: Mat4) -> Self {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        let rotation = rotation.normalize();

        Self {
            transform,
            translation: translation.extend(1.0),
            rotation: Vec4::new(rotation.x, rotation.y, rotation.z, rotation.w),
            scale: scale.extend(0.0),
        }
    }

    pub fn update(&mut self, transform: Option<Mat4>) {
        let transform = transform.unwrap_or(self.transform);
        *self = Self::new(transform);
    }
}

#[derive(Debug, Clone)]
pub struct ModelRecord {
    pub data: ModelComponent,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer_mask: u32,
    pub is_dirty: bool,
}

impl ModelRecord {
    pub fn new(
        data: ModelComponent,
        geometry_id: u32,
        material_id: Option<u32>,
        layer_mask: u32,
    ) -> Self {
        Self {
            data,
            geometry_id,
            material_id,
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
