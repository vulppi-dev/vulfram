struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    ground_color: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    intensity_range: vec2<f32>,
    spot_inner_outer: vec2<f32>,
    kind_flags: vec2<u32>,
    shadow_index: u32,
    _padding: u32,
};

struct LightCullParams {
    light_count: u32,
    camera_count: u32,
    max_lights_per_camera: u32,
    _padding: u32,
};

@group(0) @binding(0) var<storage, read> lights: array<Light>;
@group(0) @binding(1) var<storage, read_write> visible_indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> visible_counts: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> params: LightCullParams;
@group(0) @binding(4) var<storage, read> frustum_planes: array<vec4<f32>>;

const PLANES_PER_CAMERA: u32 = 6u;

fn light_visible_for_camera(light: Light, camera_index: u32) -> bool {
    let kind = light.kind_flags.x;
    if (kind == 0u || kind == 3u || kind == 4u) {
        return true;
    }

    let radius = light.intensity_range.y;
    if (radius <= 0.0) {
        return false;
    }

    let center = light.position.xyz;
    let base = camera_index * PLANES_PER_CAMERA;

    for (var i = 0u; i < PLANES_PER_CAMERA; i = i + 1u) {
        let plane = frustum_planes[base + i];
        let dist = dot(plane.xyz, center) + plane.w;
        if (dist < -radius) {
            return false;
        }
    }

    return true;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let light_idx = gid.x;
    if (light_idx >= params.light_count) {
        return;
    }

    let light = lights[light_idx];
    var camera = 0u;
    loop {
        if (camera >= params.camera_count) {
            break;
        }

        if (light_visible_for_camera(light, camera)) {
            let base = camera * params.max_lights_per_camera;
            let slot = atomicAdd(&visible_counts[camera], 1u);
            if (slot < params.max_lights_per_camera) {
                visible_indices[base + slot] = light_idx;
            }
        }

        camera = camera + 1u;
    }
}
