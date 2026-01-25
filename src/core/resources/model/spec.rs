use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec4, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct ModelComponent {
    pub transform: Mat4,
    pub translation: Vec4,
    pub rotation: Vec4,
    pub scale: Vec4,
    pub flags: UVec4, // x: flags (bit 0: receive_shadow), y: bone_offset, z: bone_count
}

impl ModelComponent {
    pub const FLAG_RECEIVE_SHADOW: u32 = 1 << 0;

    pub fn new(transform: Mat4, receive_shadow: bool) -> Self {
        Self::new_with_skin(transform, receive_shadow, 0, 0)
    }

    pub fn new_with_skin(
        transform: Mat4,
        receive_shadow: bool,
        bone_offset: u32,
        bone_count: u32,
    ) -> Self {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        let rotation = rotation.normalize();

        let mut flags = 0u32;
        if receive_shadow {
            flags |= Self::FLAG_RECEIVE_SHADOW;
        }

        Self {
            transform,
            translation: translation.extend(1.0),
            rotation: Vec4::new(rotation.x, rotation.y, rotation.z, rotation.w),
            scale: scale.extend(0.0),
            flags: UVec4::new(flags, bone_offset, bone_count, 0),
        }
    }

    pub fn update(&mut self, transform: Option<Mat4>, receive_shadow: Option<bool>) {
        let transform = transform.unwrap_or(self.transform);
        let receive_shadow =
            receive_shadow.unwrap_or((self.flags.x & Self::FLAG_RECEIVE_SHADOW) != 0);
        let bone_offset = self.flags.y;
        let bone_count = self.flags.z;
        *self = Self::new_with_skin(transform, receive_shadow, bone_offset, bone_count);
    }

    pub fn set_skinning(&mut self, bone_offset: u32, bone_count: u32) {
        self.flags.y = bone_offset;
        self.flags.z = bone_count;
    }

}

#[derive(Debug, Clone)]
pub struct ModelRecord {
    pub label: Option<String>,
    pub data: ModelComponent,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer_mask: u32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub is_dirty: bool,
}

impl ModelRecord {
    pub fn new(
        label: Option<String>,
        data: ModelComponent,
        geometry_id: u32,
        material_id: Option<u32>,
        layer_mask: u32,
        cast_shadow: bool,
        receive_shadow: bool,
    ) -> Self {
        Self {
            label,
            data,
            geometry_id,
            material_id,
            layer_mask,
            cast_shadow,
            receive_shadow,
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
