use bytemuck;
use glam::Vec3;

use crate::core::resources::geometry::primitives::PlaneOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

use super::push_face_grid;

pub fn generate_plane(options: &PlaneOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let half_x = options.size.x / 2.0;
    let half_y = options.size.y / 2.0;
    let _half_z = options.size.z / 2.0;
    let subdivisions = options.subdivisions.max(1);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut indices = Vec::new();

    push_face_grid(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut tangents,
        &mut indices,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::X,
        Vec3::Y,
        half_x,
        half_y,
        subdivisions,
    );

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
