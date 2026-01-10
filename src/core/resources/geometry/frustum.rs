use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_projection(view_projection: Mat4) -> Self {
        let m = view_projection.to_cols_array_2d();
        let row0 = Vec4::new(m[0][0], m[1][0], m[2][0], m[3][0]);
        let row1 = Vec4::new(m[0][1], m[1][1], m[2][1], m[3][1]);
        let row2 = Vec4::new(m[0][2], m[1][2], m[2][2], m[3][2]);
        let row3 = Vec4::new(m[0][3], m[1][3], m[2][3], m[3][3]);

        let planes = [
            normalize_plane(row3 + row0), // left
            normalize_plane(row3 - row0), // right
            normalize_plane(row3 + row1), // bottom
            normalize_plane(row3 - row1), // top
            normalize_plane(row3 - row2), // near (Reverse Z: Z <= W)
            normalize_plane(row2),        // far (Reverse Z: Z >= 0)
        ];

        Self { planes }
    }

    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let mut p = min;
            if plane.x >= 0.0 {
                p.x = max.x;
            }
            if plane.y >= 0.0 {
                p.y = max.y;
            }
            if plane.z >= 0.0 {
                p.z = max.z;
            }

            // Plane equation: ax + by + cz + d = 0
            // Test if the most positive point is on the negative side of the plane
            if plane.x * p.x + plane.y * p.y + plane.z * p.z + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}

fn normalize_plane(plane: Vec4) -> Vec4 {
    let normal = plane.truncate();
    let len = normal.length();
    if len > 0.0 { plane / len } else { plane }
}
