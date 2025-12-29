use crate::core::resources::vertex::GeometryPrimitiveType;
use bytemuck;
use glam::{Vec2, Vec3, Vec4};

pub fn generate_cube() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let positions = vec![
        // Front
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        // Back
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        // Top
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, -0.5),
        // Bottom
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        // Right
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        // Left
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
    ];

    let normals = vec![
        // Front
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        // Back
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, -1.0),
        // Top
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        // Bottom
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        // Right
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        // Left
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
    ];

    let uvs = vec![
        // Front
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
        // Back
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        // Top
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        // Bottom
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        // Right
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        // Left
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];

    let tangents = vec![
        // Front
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        // Back
        Vec4::new(-1.0, 0.0, 0.0, 1.0),
        Vec4::new(-1.0, 0.0, 0.0, 1.0),
        Vec4::new(-1.0, 0.0, 0.0, 1.0),
        Vec4::new(-1.0, 0.0, 0.0, 1.0),
        // Top
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        // Bottom
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        // Right
        Vec4::new(0.0, 0.0, -1.0, 1.0),
        Vec4::new(0.0, 0.0, -1.0, 1.0),
        Vec4::new(0.0, 0.0, -1.0, 1.0),
        Vec4::new(0.0, 0.0, -1.0, 1.0),
        // Left
        Vec4::new(0.0, 0.0, 1.0, 1.0),
        Vec4::new(0.0, 0.0, 1.0, 1.0),
        Vec4::new(0.0, 0.0, 1.0, 1.0),
        Vec4::new(0.0, 0.0, 1.0, 1.0),
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3, // Front
        4, 5, 6, 4, 6, 7, // Back
        8, 9, 10, 8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

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

pub fn generate_plane() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let positions = vec![
        Vec3::new(-0.5, -0.5, 0.0),
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.5, 0.5, 0.0),
        Vec3::new(-0.5, 0.5, 0.0),
    ];

    let normals = vec![
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];

    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];

    let tangents = vec![
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
    ];

    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

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

pub fn generate_pyramid() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let positions = vec![
        // Base
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        // Front face
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.0, 0.5, 0.0),
        // Right face
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.0, 0.5, 0.0),
        // Back face
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.0, 0.5, 0.0),
        // Left face
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.0, 0.5, 0.0),
    ];

    let normals = vec![
        // Base
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        // Front face
        Vec3::new(0.0, 0.5, 1.0).normalize(),
        Vec3::new(0.0, 0.5, 1.0).normalize(),
        Vec3::new(0.0, 0.5, 1.0).normalize(),
        // Right face
        Vec3::new(1.0, 0.5, 0.0).normalize(),
        Vec3::new(1.0, 0.5, 0.0).normalize(),
        Vec3::new(1.0, 0.5, 0.0).normalize(),
        // Back face
        Vec3::new(0.0, 0.5, -1.0).normalize(),
        Vec3::new(0.0, 0.5, -1.0).normalize(),
        Vec3::new(0.0, 0.5, -1.0).normalize(),
        // Left face
        Vec3::new(-1.0, 0.5, 0.0).normalize(),
        Vec3::new(-1.0, 0.5, 0.0).normalize(),
        Vec3::new(-1.0, 0.5, 0.0).normalize(),
    ];

    let uvs = vec![
        // Base
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
        // Front face
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
        // Right face
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
        // Back face
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
        // Left face
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
    ];

    let tangents = vec![
        // Base
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        // Faces
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        Vec4::new(1.0, 0.0, 0.0, 1.0),
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3, // Base
        4, 5, 6, // Front
        7, 8, 9, // Right
        10, 11, 12, // Back
        13, 14, 15, // Left
    ];

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

pub fn generate_sphere() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let radius = 0.5;
    let sectors = 32;
    let stacks = 16;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / stacks as f32 * i as f32;
        let y = radius * stack_angle.cos();
        let r = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = std::f32::consts::PI * 2.0 / sectors as f32 * j as f32;
            let x = r * sector_angle.cos();
            let z = r * sector_angle.sin();

            let pos = Vec3::new(x, y, z);
            positions.push(pos);
            normals.push(pos.normalize());
            uvs.push(Vec2::new(
                j as f32 / sectors as f32,
                i as f32 / stacks as f32,
            ));

            // Simplified tangent calculation
            let tangent = Vec3::new(-sector_angle.sin(), 0.0, sector_angle.cos()).normalize();
            tangents.push(Vec4::new(tangent.x, tangent.y, tangent.z, 1.0));
        }
    }

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _ in 0..sectors {
            if i != 0 {
                indices.push(k1 as u32);
                indices.push(k2 as u32);
                indices.push(k1 as u32 + 1);
            }

            if i != (stacks - 1) {
                indices.push(k1 as u32 + 1);
                indices.push(k2 as u32);
                indices.push(k2 as u32 + 1);
            }
            k1 += 1;
            k2 += 1;
        }
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

pub fn generate_cylinder() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let radius = 0.5;
    let height = 1.0;
    let sectors = 32;

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
        let normal = Vec3::new(x, 0.0, z).normalize();

        let pos_top = Vec3::new(x, height / 2.0, z);
        positions.push(pos_top);
        normals.push(normal);
        uvs.push(Vec2::new(i as f32 / sectors as f32, 1.0));
        let tangent_top = Vec3::new(-pos_top.z, 0.0, pos_top.x).normalize();
        tangents.push(Vec4::new(tangent_top.x, tangent_top.y, tangent_top.z, 1.0));

        let pos_bottom = Vec3::new(x, -height / 2.0, z);
        positions.push(pos_bottom);
        normals.push(normal);
        uvs.push(Vec2::new(i as f32 / sectors as f32, 0.0));
        let tangent_bottom = Vec3::new(-pos_bottom.z, 0.0, pos_bottom.x).normalize();
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

    // Top cap
    let top_center_index = positions.len() as u32;
    let top_pos = Vec3::new(0.0, height / 2.0, 0.0);
    positions.push(top_pos);
    normals.push(Vec3::new(0.0, 1.0, 0.0));
    uvs.push(Vec2::new(0.5, 0.5));
    let top_tangent = Vec3::new(-top_pos.z, 0.0, top_pos.x).normalize();
    tangents.push(Vec4::new(top_tangent.x, top_tangent.y, top_tangent.z, 1.0));

    for i in 0..=sectors {
        let angle = std::f32::consts::PI * 2.0 / sectors as f32 * i as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let pos = Vec3::new(x, height / 2.0, z);
        positions.push(pos);
        normals.push(Vec3::new(0.0, 1.0, 0.0));
        uvs.push(Vec2::new(x + 0.5, z + 0.5));
        let tangent = Vec3::new(-pos.z, 0.0, pos.x).normalize();
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
    let bottom_tangent = Vec3::new(-bottom_pos.z, 0.0, bottom_pos.x).normalize();
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
        uvs.push(Vec2::new(x + 0.5, z + 0.5));
        let tangent = Vec3::new(-pos.z, 0.0, pos.x).normalize();
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

pub fn generate_torus() -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let major_radius = 0.4;
    let minor_radius = 0.1;
    let major_segments = 32;
    let minor_segments = 16;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
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
            normals.push(normal);

            uvs.push(Vec2::new(
                i as f32 / major_segments as f32,
                j as f32 / minor_segments as f32,
            ));

            tangents.push(Vec4::new(-major_sin, 0.0, major_cos, 1.0));
        }
    }

    for i in 0..major_segments {
        let mut i1 = i * (minor_segments + 1);
        let mut i2 = (i + 1) * (minor_segments + 1);
        for j in 0..minor_segments {
            indices.push(i1 as u32);
            indices.push(i2 as u32);
            indices.push((i1 + 1) as u32);

            indices.push((i1 + 1) as u32);
            indices.push(i2 as u32);
            indices.push((i2 + 1) as u32);
            i1 += 1;
            i2 += 1;
        }
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
