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
    flags: vec4<u32>, // x: flags (bit 0: receive_shadow)
}

@group(0) @binding(1) var<uniform> camera: Camera;
@group(1) @binding(0) var<storage, read> models: array<Model>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_id: u32) -> @builtin(position) vec4<f32> {
    let model = models[instance_id];
    let world_pos = model.transform * vec4<f32>(in.position, 1.0);
    return camera.view_projection * world_pos;
}
