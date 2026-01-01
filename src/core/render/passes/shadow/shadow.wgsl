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
@group(1) @binding(0) var<uniform> model: Model;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    // Camera here will be the Light's View-Projection for the specific page
    return camera.view_projection * model.transform * vec4<f32>(in.position, 1.0);
}
