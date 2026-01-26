use bytemuck;
use glam::{Vec2, Vec3};

use crate::core::resources::geometry::primitives::TorusOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

use super::compute_tangents;

pub fn generate_torus(options: &TorusOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let major_radius = options.major_radius;
    let minor_radius = options.minor_radius;
    let major_segments = options.major_segments.max(3);
    let minor_segments = options.minor_segments.max(3);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segments {
        let major_angle = std::f32::consts::PI * 2.0 / major_segments as f32 * i as f32;
        let major_cos = major_angle.cos();
        let major_sin = major_angle.sin();

        for j in 0..=minor_segments {
            let minor_angle = std::f32::consts::PI * 2.0 / minor_segments as f32 * j as f32;
            let minor_cos = minor_angle.cos();
            let minor_sin = minor_angle.sin();

            let x = (major_radius + minor_radius * minor_cos) * major_cos;
            let y = minor_radius * minor_sin;
            let z = (major_radius + minor_radius * minor_cos) * major_sin;
            positions.push(Vec3::new(x, y, z));

            let normal = Vec3::new(minor_cos * major_cos, minor_sin, minor_cos * major_sin);
            normals.push(normal.normalize_or_zero());

            uvs.push(Vec2::new(
                i as f32 / major_segments as f32,
                j as f32 / minor_segments as f32,
            ));

        }
    }

    for i in 0..major_segments {
        let mut i1 = i * (minor_segments + 1);
        let mut i2 = (i + 1) * (minor_segments + 1);
        for _ in 0..minor_segments {
            indices.push(i1 as u32);
            indices.push((i1 + 1) as u32);
            indices.push(i2 as u32);

            indices.push((i1 + 1) as u32);
            indices.push((i2 + 1) as u32);
            indices.push(i2 as u32);
            i1 += 1;
            i2 += 1;
        }
    }
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
