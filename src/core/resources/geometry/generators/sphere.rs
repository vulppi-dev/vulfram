use bytemuck;
use glam::{Vec2, Vec3, Vec4};

use crate::core::resources::geometry::primitives::SphereOptions;
use crate::core::resources::vertex::GeometryPrimitiveType;

pub fn generate_sphere(options: &SphereOptions) -> Vec<(GeometryPrimitiveType, Vec<u8>)> {
    let radius = options.radius;
    let sectors = options.sectors.max(3);
    let stacks = options.stacks.max(2);

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
            normals.push(pos.normalize_or_zero());
            uvs.push(Vec2::new(
                j as f32 / sectors as f32,
                i as f32 / stacks as f32,
            ));

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
