use bytemuck;
use glam::{Vec2, Vec3, Vec4};

use crate::core::resources::geometry::primitives::CylinderOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

pub fn generate_cylinder(options: &CylinderOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let radius = options.radius;
    let height = options.height;
    let sectors = options.sectors.max(3);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut indices = Vec::new();

    // Side
    for i in 0..=sectors {
        let angle = std::f32::consts::PI * 2.0 / sectors as f32 * i as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let normal = Vec3::new(x, 0.0, z).normalize_or_zero();

        let pos_top = Vec3::new(x, height / 2.0, z);
        positions.push(pos_top);
        normals.push(normal);
        uvs.push(Vec2::new(i as f32 / sectors as f32, 1.0));
        let tangent_top = Vec3::new(-pos_top.z, 0.0, pos_top.x).normalize_or_zero();
        tangents.push(Vec4::new(tangent_top.x, tangent_top.y, tangent_top.z, 1.0));

        let pos_bottom = Vec3::new(x, -height / 2.0, z);
        positions.push(pos_bottom);
        normals.push(normal);
        uvs.push(Vec2::new(i as f32 / sectors as f32, 0.0));
        let tangent_bottom = Vec3::new(-pos_bottom.z, 0.0, pos_bottom.x).normalize_or_zero();
        tangents.push(Vec4::new(
            tangent_bottom.x,
            tangent_bottom.y,
            tangent_bottom.z,
            1.0,
        ));
    }

    for i in 0..sectors {
        let i0 = i * 2;
        let i1 = i0 + 1;
        let i2 = (i + 1) * 2;
        let i3 = i2 + 1;
        indices.push(i0 as u32);
        indices.push(i2 as u32);
        indices.push(i1 as u32);
        indices.push(i1 as u32);
        indices.push(i2 as u32);
        indices.push(i3 as u32);
    }

    let uv_scale = if radius.abs() > 0.0 {
        1.0 / (2.0 * radius)
    } else {
        0.0
    };

    // Top cap
    let top_center_index = positions.len() as u32;
    let top_pos = Vec3::new(0.0, height / 2.0, 0.0);
    positions.push(top_pos);
    normals.push(Vec3::new(0.0, 1.0, 0.0));
    uvs.push(Vec2::new(0.5, 0.5));
    let top_tangent = Vec3::new(-top_pos.z, 0.0, top_pos.x).normalize_or_zero();
    tangents.push(Vec4::new(top_tangent.x, top_tangent.y, top_tangent.z, 1.0));

    for i in 0..=sectors {
        let angle = std::f32::consts::PI * 2.0 / sectors as f32 * i as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let pos = Vec3::new(x, height / 2.0, z);
        positions.push(pos);
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        uvs.push(Vec2::new(x * uv_scale + 0.5, z * uv_scale + 0.5));
        let tangent = Vec3::new(-pos.z, 0.0, pos.x).normalize_or_zero();
        tangents.push(Vec4::new(tangent.x, tangent.y, tangent.z, 1.0));
    }

    for i in 0..sectors {
        indices.push(top_center_index);
        indices.push(top_center_index + i as u32 + 1);
        indices.push(top_center_index + i as u32 + 2);
    }

    // Bottom cap
    let bottom_center_index = positions.len() as u32;
    let bottom_pos = Vec3::new(0.0, -height / 2.0, 0.0);
    positions.push(bottom_pos);
    normals.push(Vec3::new(0.0, -1.0, 0.0));
    uvs.push(Vec2::new(0.5, 0.5));
    let bottom_tangent = Vec3::new(-bottom_pos.z, 0.0, bottom_pos.x).normalize_or_zero();
    tangents.push(Vec4::new(
        bottom_tangent.x,
        bottom_tangent.y,
        bottom_tangent.z,
        1.0,
    ));

    for i in 0..=sectors {
        let angle = std::f32::consts::PI * 2.0 / sectors as f32 * i as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let pos = Vec3::new(x, -height / 2.0, z);
        positions.push(pos);
        normals.push(Vec3::new(0.0, -1.0, 0.0));
        uvs.push(Vec2::new(x * uv_scale + 0.5, z * uv_scale + 0.5));
        let tangent = Vec3::new(-pos.z, 0.0, pos.x).normalize_or_zero();
        tangents.push(Vec4::new(tangent.x, tangent.y, tangent.z, 1.0));
    }

    for i in 0..sectors {
        indices.push(bottom_center_index);
        indices.push(bottom_center_index + i as u32 + 2);
        indices.push(bottom_center_index + i as u32 + 1);
    }

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
