pub fn build_skinned_plane(
    grid_x: u32,
    grid_z: u32,
    size: f32,
    bone_count: u32,
) -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    Vec<[u16; 4]>,
    Vec<[f32; 4]>,
    Vec<u32>,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    let step_x = 1.0 / grid_x as f32;
    let step_z = 1.0 / grid_z as f32;
    let half = size * 0.5;
    let segments = (bone_count - 1).max(1) as f32;
    let smoothstep = |t: f32| t * t * (3.0 - 2.0 * t);

    for z in 0..=grid_z {
        for x in 0..=grid_x {
            let fx = x as f32 * step_x;
            let fz = z as f32 * step_z;
            let pos_x = fx * size - half;
            let pos_z = fz * size - half;

            positions.push([pos_x, 0.0, pos_z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([fx, fz]);

            let bone_f = fx * segments;
            let bone_idx = bone_f.floor() as u32;
            let next_idx = (bone_idx + 1).min(bone_count - 1);
            let t = smoothstep(bone_f - bone_idx as f32);
            joints.push([bone_idx as u16, next_idx as u16, 0, 0]);
            weights.push([1.0 - t, t, 0.0, 0.0]);
        }
    }

    let verts_x = grid_x + 1;
    for z in 0..grid_z {
        for x in 0..grid_x {
            let i0 = z * verts_x + x;
            let i1 = i0 + 1;
            let i2 = i0 + verts_x;
            let i3 = i2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    (positions, normals, uvs, joints, weights, indices)
}
