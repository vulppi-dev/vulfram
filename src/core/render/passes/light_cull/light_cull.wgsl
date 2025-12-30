struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    intensity_range: vec2<f32>,
    spot_inner_outer: vec2<f32>,
    kind_flags: vec2<u32>,
    _padding: vec2<u32>,
};

struct LightCullParams {
    light_count: u32,
    camera_count: u32,
    max_lights_per_camera: u32,
    _padding: u32,
};

@group(0) @binding(0) var<storage, read> lights: array<Light>;
@group(0) @binding(1) var<storage, read_write> visible_indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> visible_counts: array<u32>;
@group(0) @binding(3) var<uniform> params: LightCullParams;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let light_idx = gid.x;
    if (light_idx >= params.light_count) {
        return;
    }

    var camera = 0u;
    loop {
        if (camera >= params.camera_count) {
            break;
        }
        let base = camera * params.max_lights_per_camera;
        visible_indices[base + light_idx] = light_idx;
        if (light_idx == 0u) {
            visible_counts[camera] = params.light_count;
        }
        camera = camera + 1u;
    }
}
