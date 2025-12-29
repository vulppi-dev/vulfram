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
