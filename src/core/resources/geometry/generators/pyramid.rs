use bytemuck;
use glam::{Vec2, Vec3};

use crate::core::resources::geometry::primitives::PyramidOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

use super::{compute_tangents, push_face_grid, push_triangle_grid};

pub fn generate_pyramid(options: &PyramidOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let half_x = options.size.x / 2.0;
    let half_y = options.size.y / 2.0;
    let half_z = options.size.z / 2.0;
    let subdivisions = options.subdivisions.max(1);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Base (XZ plane)
    push_face_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, -half_y, 0.0),
        Vec3::X,
        Vec3::Z,
        half_x,
        half_z,
        subdivisions,
    );

    let apex = Vec3::new(0.0, half_y, 0.0);
    let front_left = Vec3::new(-half_x, -half_y, half_z);
    let front_right = Vec3::new(half_x, -half_y, half_z);
    let back_right = Vec3::new(half_x, -half_y, -half_z);
    let back_left = Vec3::new(-half_x, -half_y, -half_z);

    let uv_apex = Vec2::new(0.5, 1.0);
    let uv_left = Vec2::new(0.0, 0.0);
    let uv_right = Vec2::new(1.0, 0.0);

    // Front
    push_triangle_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        apex,
        front_left,
        front_right,
        uv_apex,
        uv_left,
        uv_right,
        subdivisions,
    );

    // Right
    push_triangle_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        apex,
        front_right,
        back_right,
        uv_apex,
        uv_left,
        uv_right,
        subdivisions,
    );

    // Back
    push_triangle_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        apex,
        back_right,
        back_left,
        uv_apex,
        uv_left,
        uv_right,
        subdivisions,
    );

    // Left
    push_triangle_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        apex,
        back_left,
        front_left,
        uv_apex,
        uv_left,
        uv_right,
        subdivisions,
    );
    let tangents = compute_tangents(&positions, &normals, &uvs, &indices);

    vec![
        (
            GeometryPrimitiveType::Position,
            bytemuck::cast_slice(&positions).to_vec(),
        ),
        (
            GeometryPrimitiveType::Normal,
            bytemuck::cast_slice(&normals).to_vec(),
        ),
        (
            GeometryPrimitiveType::UV,
            bytemuck::cast_slice(&uvs).to_vec(),
        ),
        (
            GeometryPrimitiveType::Tangent,
            bytemuck::cast_slice(&tangents).to_vec(),
        ),
        (
            GeometryPrimitiveType::Index,
            bytemuck::cast_slice(&indices).to_vec(),
        ),
    ]
}
