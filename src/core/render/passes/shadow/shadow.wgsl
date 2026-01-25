struct Camera {
    position: vec4<f32>,
    direction: vec4<f32>,
    up: vec4<f32>,
    near_far: vec2<f32>,
    kind_flags: vec2<u32>,
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    view_projection: mat4x4<f32>,
}

struct Model {
    transform: mat4x4<f32>,
    translation: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    flags: vec4<u32>, // x: flags, y: bone_offset, z: bone_count
}

@group(0) @binding(1) var<uniform> camera: Camera;
@group(1) @binding(0) var<storage, read> models: array<Model>;
@group(1) @binding(1) var<storage, read> bones: array<mat4x4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(6) joints: vec4<u32>,
    @location(7) weights: vec4<f32>,
}

fn bone_at(index: u32, bone_offset: u32, bone_count: u32) -> mat4x4<f32> {
    if (index < bone_count) {
        return bones[bone_offset + index];
    }
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
}

fn skin_position(
    position: vec3<f32>,
    joints: vec4<u32>,
    weights: vec4<f32>,
    bone_offset: u32,
    bone_count: u32,
) -> vec3<f32> {
    let m0 = bone_at(joints.x, bone_offset, bone_count);
    let m1 = bone_at(joints.y, bone_offset, bone_count);
    let m2 = bone_at(joints.z, bone_offset, bone_count);
    let m3 = bone_at(joints.w, bone_offset, bone_count);
    let p = vec4<f32>(position, 1.0);
    let skinned = (m0 * p) * weights.x
        + (m1 * p) * weights.y
        + (m2 * p) * weights.z
        + (m3 * p) * weights.w;
    return skinned.xyz;
}

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_id: u32) -> @builtin(position) vec4<f32> {
    let model = models[instance_id];
    let bone_offset = model.flags.y;
    let bone_count = model.flags.z;
    var local_pos = in.position;
    if (bone_count > 0u) {
        local_pos = skin_position(in.position, in.joints, in.weights, bone_offset, bone_count);
    }
    let world_pos = model.transform * vec4<f32>(local_pos, 1.0);
    return camera.view_projection * world_pos;
}
