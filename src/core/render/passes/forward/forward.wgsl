// -----------------------------------------------------------------------------
// Structures
// -----------------------------------------------------------------------------

struct Frame {
    time: f32,
    delta_time: f32,
    frame_index: u32,
    _padding: u32,
}

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
}

// -----------------------------------------------------------------------------
// Bindings
// -----------------------------------------------------------------------------

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> model: Model;

// -----------------------------------------------------------------------------
// Input / Output
// -----------------------------------------------------------------------------

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) color0: vec4<f32>,
    @location(4) uv0: vec2<f32>,
    @location(5) uv1: vec2<f32>,
    @location(6) joints: vec4<u32>,
    @location(7) weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) color0: vec4<f32>,
}

// -----------------------------------------------------------------------------
// Vertex Shader
// -----------------------------------------------------------------------------

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Simple MVP transformation
    let world_pos = model.transform * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_projection * world_pos;

    out.world_position = world_pos.xyz;

    // Pass-through simple normal transformation (ideally uses normal matrix)
    out.normal = (model.transform * vec4<f32>(in.normal, 0.0)).xyz;

    out.uv0 = in.uv0;
    out.color0 = in.color0;

    return out;
}

// -----------------------------------------------------------------------------
// Fragment Shader
// -----------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Visualization of normals mixed with vertex color for debugging
    let n = normalize(in.normal) * 0.5 + 0.5;
    let final_color = mix(vec4<f32>(n, 1.0), in.color0, 0.2);

    return final_color;
}
