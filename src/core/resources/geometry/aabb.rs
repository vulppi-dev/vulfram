use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl Aabb {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        // Assume f32x3 (Position stream)
        let f32_count = bytes.len() / 4;
        let floats = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, f32_count) };

        for i in (0..floats.len()).step_by(3) {
            if i + 2 < floats.len() {
                let p = Vec3::new(floats[i], floats[i + 1], floats[i + 2]);
                min = min.min(p);
                max = max.max(p);
            }
        }

        Self { min, max }
    }

    pub fn transform(&self, matrix: &glam::Mat4) -> Self {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut new_min = Vec3::splat(f32::INFINITY);
        let mut new_max = Vec3::splat(f32::NEG_INFINITY);

        for &c in &corners {
            let p = matrix.transform_point3(c);
            new_min = new_min.min(p);
            new_max = new_max.max(p);
        }

        Self {
            min: new_min,
            max: new_max,
        }
    }
}
