use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum CameraKind {
    Perspective = 0,
    Orthographic,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[repr(C)]
pub struct CameraComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub up: Vec4,
    pub near_far: Vec2,
    pub kind_flags: UVec2,

    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
}

impl CameraComponent {
    /// Create from raw input data
    ///
    /// `viewport`:
    /// - If `Perspective`: x=width, y=height (for aspect ratio calculation)
    /// - If `Orthographic`: x=left, y=right, z=bottom, w=top (rect bounds)
    pub fn new(
        transform: Mat4,
        kind: CameraKind,
        flags: u32,
        near_far: Vec2,
        viewport: Vec4,
    ) -> Self {
        let position = transform.w_axis.truncate();
        let rotation = Quat::from_mat4(&transform);
        let direction = rotation * Vec3::NEG_Z;
        let up = rotation * Vec3::Y;

        let view = Mat4::look_to_rh(position, direction, up);

        let projection = match kind {
            CameraKind::Perspective => {
                let fov_y = 45.0_f32.to_radians();
                let aspect_ratio = viewport.x / viewport.y;
                Mat4::perspective_rh(fov_y, aspect_ratio, near_far.x, near_far.y)
            }
            CameraKind::Orthographic => {
                Mat4::orthographic_rh(
                    viewport.x, // left
                    viewport.y, // right
                    viewport.z, // bottom
                    viewport.w, // top
                    near_far.x, near_far.y,
                )
            }
        };

        let view_projection = projection * view;

        Self {
            position: position.extend(1.0),
            direction: direction.extend(0.0),
            up: up.extend(0.0),
            near_far,
            kind_flags: UVec2::new(kind as u32, flags),
            projection,
            view,
            view_projection,
        }
    }

    /// Update from raw input data
    pub fn update(
        &mut self,
        transform: Option<Mat4>,
        kind: Option<CameraKind>,
        flags: Option<u32>,
        near_far: Option<Vec2>,
        viewport: Vec4,
    ) {
        let transform = transform.unwrap_or_else(|| {
            let pos = self.position.truncate();
            let dir = self.direction.truncate();
            let up = self.up.truncate();
            Mat4::look_to_rh(pos, dir, up).inverse()
        });

        let kind = kind.unwrap_or_else(|| {
            if self.kind_flags.x == 0 {
                CameraKind::Perspective
            } else {
                CameraKind::Orthographic
            }
        });

        let flags = flags.unwrap_or(self.kind_flags.y);
        let near_far = near_far.unwrap_or(self.near_far);

        *self = Self::new(transform, kind, flags, near_far, viewport);
    }
}

#[derive(Debug, Clone)]
pub struct CameraRecord {
    pub data: CameraComponent,
    pub layer_mask: u32,
    pub is_dirty: bool,
}

impl CameraRecord {
    pub fn new(data: CameraComponent, layer_mask: u32) -> Self {
        Self {
            data,
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
