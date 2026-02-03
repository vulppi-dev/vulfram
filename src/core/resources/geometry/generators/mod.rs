mod cube;
mod cylinder;
mod plane;
mod pyramid;
mod sphere;
mod torus;

pub use cube::generate_cube;
pub use cylinder::generate_cylinder;
pub use plane::generate_plane;
pub use pyramid::generate_pyramid;
pub use sphere::generate_sphere;
pub use torus::generate_torus;

use glam::{Vec2, Vec3, Vec4};

pub(crate) fn push_face_grid(
    positions: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    indices: &mut Vec<u32>,
    center: Vec3,
    axis_u: Vec3,
    axis_v: Vec3,
    half_u: f32,
    half_v: f32,
    subdivisions: u32,
) {
    let subdivisions = subdivisions.max(1);
    let n = subdivisions as usize;
    let normal = axis_u.cross(axis_v).normalize();

    let base_index = positions.len() as u32;

    for y in 0..=n {
        let v = y as f32 / subdivisions as f32;
        for x in 0..=n {
            let u = x as f32 / subdivisions as f32;
            let pos =
                center + axis_u * ((u * 2.0 - 1.0) * half_u) + axis_v * ((v * 2.0 - 1.0) * half_v);

            positions.push(pos);
            normals.push(normal);
            uvs.push(Vec2::new(u, v));
        }
    }

    let row_stride = (n + 1) as u32;
    for y in 0..n {
        for x in 0..n {
            let i0 = base_index + y as u32 * row_stride + x as u32;
            let i1 = i0 + 1;
            let i2 = i0 + row_stride;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i1);
            indices.push(i2);

            indices.push(i1);
            indices.push(i3);
            indices.push(i2);
        }
    }
}

pub(crate) fn push_triangle_grid(
    positions: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    indices: &mut Vec<u32>,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    subdivisions: u32,
) {
    let subdivisions = subdivisions.max(1) as usize;
    let normal = (v1 - v0).cross(v2 - v0).normalize();

    let mut row_starts = Vec::with_capacity(subdivisions + 1);

    for row in 0..=subdivisions {
        row_starts.push(positions.len() as u32);

        let t = row as f32 / subdivisions as f32;
        let start_pos = v0.lerp(v2, t);
        let end_pos = v0.lerp(v1, t);
        let start_uv = uv0.lerp(uv2, t);
        let end_uv = uv0.lerp(uv1, t);

        for col in 0..=row {
            let s = if row == 0 {
                0.0
            } else {
                col as f32 / row as f32
            };
            let pos = start_pos.lerp(end_pos, s);
            let uv = start_uv.lerp(end_uv, s);

            positions.push(pos);
            normals.push(normal);
            uvs.push(uv);
        }
    }

    for row in 0..subdivisions {
        let row_start = row_starts[row];
        let next_row_start = row_starts[row + 1];
        let cols = row + 1;

        for col in 0..cols {
            let i0 = row_start + col as u32;
            let i1 = next_row_start + col as u32;
            let i2 = next_row_start + col as u32 + 1;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            if col + 1 < cols {
                let i3 = row_start + col as u32 + 1;
                indices.push(i0);
                indices.push(i3);
                indices.push(i2);
            }
        }
    }
}

pub(crate) fn compute_tangents(
    positions: &[Vec3],
    normals: &[Vec3],
    uvs: &[Vec2],
    indices: &[u32],
) -> Vec<Vec4> {
    let count = positions.len();
    let mut tan1 = vec![Vec3::ZERO; count];
    let mut tan2 = vec![Vec3::ZERO; count];

    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= count || i1 >= count || i2 >= count {
            continue;
        }

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let uv0 = uvs[i0];
        let uv1 = uvs[i1];
        let uv2 = uvs[i2];

        let delta_pos1 = p1 - p0;
        let delta_pos2 = p2 - p0;
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let denom = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;
        if denom.abs() < 1e-6 {
            continue;
        }
        let r = 1.0 / denom;
        let sdir = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        let tdir = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

        tan1[i0] += sdir;
        tan1[i1] += sdir;
        tan1[i2] += sdir;

        tan2[i0] += tdir;
        tan2[i1] += tdir;
        tan2[i2] += tdir;
    }

    let mut tangents = Vec::with_capacity(count);
    for i in 0..count {
        let n = normals
            .get(i)
            .copied()
            .unwrap_or(Vec3::Y)
            .normalize_or_zero();
        let t = tan1[i];
        let t = if t.length_squared() < 1e-8 {
            if n.y.abs() < 0.999 {
                n.cross(Vec3::Y).normalize_or_zero()
            } else {
                n.cross(Vec3::X).normalize_or_zero()
            }
        } else {
            (t - n * n.dot(t)).normalize_or_zero()
        };
        let w = if n.cross(t).dot(tan2[i]) < 0.0 {
            -1.0
        } else {
            1.0
        };
        tangents.push(Vec4::new(t.x, t.y, t.z, w));
    }

    tangents
}
