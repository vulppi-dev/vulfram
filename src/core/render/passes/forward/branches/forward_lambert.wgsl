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

struct LightDrawParams {
    camera_index: u32,
    max_lights_per_camera: u32,
};

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
}

struct ShadowPageEntry {
    scale_offset: vec4<f32>,
    layer_index: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

struct ShadowParams {
    virtual_grid_size: f32,
    pcf_range: i32,
    table_capacity: u32,
    bias_min: f32,
    bias_slope: f32,
    point_bias_min: f32,
    point_bias_slope: f32,
    _padding: f32,
}

struct Model {
    transform: mat4x4<f32>,
    translation: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    flags: vec4<u32>,
}

struct MaterialLambert {
    base_color: vec4<f32>,
}

// -----------------------------------------------------------------------------
// Bindings
// -----------------------------------------------------------------------------

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> light_params: LightDrawParams;
@group(0) @binding(3) var<storage, read> lights: array<Light>;
@group(0) @binding(4) var<storage, read> visible_indices: array<u32>;
@group(0) @binding(5) var<storage, read> visible_counts: array<u32>;
@group(0) @binding(6) var<uniform> shadow_params: ShadowParams;
@group(0) @binding(7) var shadow_atlas: texture_depth_2d_array;
@group(0) @binding(8) var<storage, read> shadow_page_table: array<ShadowPageEntry>;
@group(0) @binding(9) var point_clamp_sampler: sampler;
@group(0) @binding(10) var linear_clamp_sampler: sampler;
@group(0) @binding(11) var point_repeat_sampler: sampler;
@group(0) @binding(12) var linear_repeat_sampler: sampler;
@group(0) @binding(13) var shadow_sampler: sampler_comparison;
@group(0) @binding(14) var<storage, read> point_light_vp: array<mat4x4<f32>>;

@group(1) @binding(0) var<uniform> model: Model;
@group(1) @binding(1) var<uniform> material: MaterialLambert;

// -----------------------------------------------------------------------------
// Vertex I/O
// -----------------------------------------------------------------------------

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(3) color0: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color0: vec4<f32>,
}

// -----------------------------------------------------------------------------
// Shadow sampling helpers (ported from legacy forward shader)
// -----------------------------------------------------------------------------

fn compute_table_id(light_base: u32, grid_x: u32, grid_y: u32, grid_size_u: u32) -> u32 {
    let grid_area = grid_size_u * grid_size_u;
    let linear_id = light_base * grid_area + grid_y * grid_size_u + grid_x;
    return linear_id % shadow_params.table_capacity;
}

fn sample_shadow_page_at(
    light_base: u32,
    light_ndc: vec3<f32>,
    ndotl: f32,
    bias_min: f32,
    bias_slope: f32
) -> f32 {
    let light_uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, light_ndc.y * -0.5 + 0.5);
    let light_depth = light_ndc.z;

    if (light_uv.x < 0.0 || light_uv.x > 1.0 || light_uv.y < 0.0 || light_uv.y > 1.0 || light_depth < 0.0 || light_depth > 1.0) {
        return 1.0;
    }

    let grid_size_f = shadow_params.virtual_grid_size;
    let grid_size_u = u32(grid_size_f);
    let grid_x = u32(clamp(light_uv.x * grid_size_f, 0.0, grid_size_f - 1.0));
    let grid_y = u32(clamp(light_uv.y * grid_size_f, 0.0, grid_size_f - 1.0));

    let table_id = compute_table_id(light_base, grid_x, grid_y, grid_size_u);
    let page = shadow_page_table[table_id];

    if (page.scale_offset.x == 0.0 && page.scale_offset.y == 0.0) {
        return 1.0;
    }

    let page_origin = vec2<f32>(f32(grid_x), f32(grid_y)) / grid_size_f;
    let page_uv = (light_uv - page_origin) * grid_size_f;

    if (page_uv.x < 0.0 || page_uv.x > 1.0 || page_uv.y < 0.0 || page_uv.y > 1.0) {
        return 1.0;
    }

    let atlas_uv_center = (page_uv * page.scale_offset.xy) + page.scale_offset.zw;
    let bias = max(bias_min, bias_slope * (1.0 - ndotl));

    let dim = textureDimensions(shadow_atlas);
    let atlas_texel = 1.0 / vec2<f32>(f32(dim.x), f32(dim.y));
    let tile_min = page.scale_offset.zw;
    let tile_max = page.scale_offset.zw + page.scale_offset.xy;
    let guard = atlas_texel * 1.5;
    let uv_min = tile_min + guard;
    let uv_max = tile_max - guard;

    if (shadow_params.pcf_range == 0) {
        let uv = clamp(atlas_uv_center, uv_min, uv_max);
        return textureSampleCompare(
            shadow_atlas,
            shadow_sampler,
            uv,
            i32(page.layer_index),
            light_depth - bias
        );
    }

    var sum = 0.0;
    var samples = 0.0;
    let range = shadow_params.pcf_range;

    for (var oy = -range; oy <= range; oy = oy + 1) {
        for (var ox = -range; ox <= range; ox = ox + 1) {
            let offset = vec2<f32>(f32(ox), f32(oy)) * atlas_texel;
            let uv = clamp(atlas_uv_center + offset, uv_min, uv_max);
            sum += textureSampleCompare(
                shadow_atlas,
                shadow_sampler,
                uv,
                i32(page.layer_index),
                light_depth - bias
            );
            samples += 1.0;
        }
    }

    return sum / samples;
}

fn dir_component(v: vec3<f32>, axis: u32) -> f32 {
    if (axis == 0u) { return v.x; }
    if (axis == 1u) { return v.y; }
    return v.z;
}

fn face_from_axis(axis: u32, comp: f32) -> u32 {
    let pos = comp >= 0.0;
    if (axis == 0u) { return select(1u, 0u, pos); }
    if (axis == 1u) { return select(3u, 2u, pos); }
    return select(5u, 4u, pos);
}

fn sample_point_face(light: Light, face: u32, world_pos: vec3<f32>, ndotl: f32) -> f32 {
    let shadow_idx = light.shadow_index;
    let vp = point_light_vp[shadow_idx * 6u + face];
    let clip = vp * vec4<f32>(world_pos, 1.0);
    let ndc = clip.xyz / clip.w;
    if (ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 || ndc.z < 0.0 || ndc.z > 1.0) {
        return 1.0;
    }
    let light_base = shadow_idx * 6u + face;
    return sample_shadow_page_at(light_base, ndc, ndotl, shadow_params.point_bias_min, shadow_params.point_bias_slope);
}

fn point_shadow_factor(light: Light, world_pos: vec3<f32>, ndotl: f32) -> f32 {
    let v = world_pos - light.position.xyz;
    let dist = length(v);
    if (dist <= 1e-5) { return 1.0; }
    let dir = v / dist;
    let ad = abs(dir);
    var a0: u32 = 0u;
    var m0: f32 = ad.x;
    var a1: u32 = 1u;
    var m1: f32 = ad.y;
    if (m1 > m0) {
        let tmpa = a0; a0 = a1; a1 = tmpa;
        let tmpm = m0; m0 = m1; m1 = tmpm;
    }
    if (ad.z > m0) {
        a1 = a0; m1 = m0;
        a0 = 2u; m0 = ad.z;
    } else if (ad.z > m1) {
        a1 = 2u; m1 = ad.z;
    }
    let face0 = face_from_axis(a0, dir_component(dir, a0));
    let face1 = face_from_axis(a1, dir_component(dir, a1));
    let s0 = sample_point_face(light, face0, world_pos, ndotl);
    let s1 = sample_point_face(light, face1, world_pos, ndotl);
    return min(s0, s1);
}

fn get_shadow_factor(light: Light, world_pos: vec3<f32>, ndotl: f32) -> f32 {
    let model_receive_shadow = (model.flags.x & 1u) != 0u;
    let light_cast_shadow = (light.kind_flags.y & 1u) != 0u;
    if (!model_receive_shadow || !light_cast_shadow) { return 1.0; }
    if (ndotl <= 0.0) { return 1.0; }
    if (light.kind_flags.x == 1u) { return point_shadow_factor(light, world_pos, ndotl); }
    let light_clip = light.view_projection * vec4<f32>(world_pos, 1.0);
    let light_ndc = light_clip.xyz / light_clip.w;
    let shadow_idx = light.shadow_index;
    let light_base = shadow_idx * 6u + 0u;
    return sample_shadow_page_at(light_base, light_ndc, ndotl, shadow_params.bias_min, shadow_params.bias_slope);
}

// -----------------------------------------------------------------------------
// Lighting (Lambert)
// -----------------------------------------------------------------------------

fn calculate_directional_light(light: Light, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let l = normalize(-light.direction.xyz);
    let ndotl = max(dot(normal, l), 0.0);
    if (ndotl <= 0.0) { return vec3<f32>(0.0); }
    let shadow = get_shadow_factor(light, world_pos, ndotl);
    return light.color.rgb * light.intensity_range.x * ndotl * shadow;
}

fn calculate_ambient_light(light: Light) -> vec3<f32> {
    return light.color.rgb * light.intensity_range.x;
}

fn calculate_hemisphere_light(light: Light, normal: vec3<f32>) -> vec3<f32> {
    let up = normalize(light.direction.xyz);
    let w = dot(normal, up) * 0.5 + 0.5;
    return mix(light.ground_color.rgb, light.color.rgb, w) * light.intensity_range.x;
}

fn calculate_spot_light(light: Light, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let light_to_pos = world_pos - light.position.xyz;
    let dist = length(light_to_pos);
    let l = normalize(-light_to_pos);
    let range = light.intensity_range.y;
    if (dist > range) { return vec3<f32>(0.0); }
    let attenuation = pow(clamp(1.0 - dist / range, 0.0, 1.0), 2.0);
    let theta = dot(l, normalize(-light.direction.xyz));
    let inner = cos(light.spot_inner_outer.x);
    let outer = cos(light.spot_inner_outer.y);
    let epsilon = inner - outer;
    let spot_intensity = clamp((theta - outer) / epsilon, 0.0, 1.0);
    let ndotl = max(dot(normal, l), 0.0);
    if (ndotl <= 0.0) { return vec3<f32>(0.0); }
    let shadow = get_shadow_factor(light, world_pos, ndotl);
    return light.color.rgb * light.intensity_range.x * ndotl * attenuation * spot_intensity * shadow;
}

fn calculate_point_light(light: Light, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let light_to_pos = world_pos - light.position.xyz;
    let dist = length(light_to_pos);
    let l = normalize(-light_to_pos);
    let range = light.intensity_range.y;
    if (dist > range) { return vec3<f32>(0.0); }
    let attenuation = pow(clamp(1.0 - dist / range, 0.0, 1.0), 2.0);
    let ndotl = max(dot(normal, l), 0.0);
    if (ndotl <= 0.0) { return vec3<f32>(0.0); }
    let shadow = get_shadow_factor(light, world_pos, ndotl);
    return light.color.rgb * light.intensity_range.x * ndotl * attenuation * shadow;
}

// -----------------------------------------------------------------------------
// Vertex
// -----------------------------------------------------------------------------

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.transform * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_projection * world_pos;
    out.world_position = world_pos.xyz;
    out.normal = (model.transform * vec4<f32>(in.normal, 0.0)).xyz;
    out.color0 = in.color0;
    return out;
}

// -----------------------------------------------------------------------------
// Fragment
// -----------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    var color = material.base_color.rgb * in.color0.rgb;

    let cam = light_params.camera_index;
    let base = cam * light_params.max_lights_per_camera;
    let count = min(visible_counts[cam], light_params.max_lights_per_camera);

    if (count > 0u) {
        var lighting = vec3<f32>(0.0);
        for (var i = 0u; i < count; i++) {
            let idx = visible_indices[base + i];
            let light = lights[idx];
            switch (light.kind_flags.x) {
                case 0u: { lighting += calculate_directional_light(light, n, in.world_position); }
                case 1u: { lighting += calculate_point_light(light, n, in.world_position); }
                case 2u: { lighting += calculate_spot_light(light, n, in.world_position); }
                case 3u: { lighting += calculate_ambient_light(light); }
                case 4u: { lighting += calculate_hemisphere_light(light, n); }
                default: { }
            }
        }
        color *= (lighting + vec3<f32>(0.001));
    } else {
        color *= vec3<f32>(0.001);
    }

    return vec4<f32>(color, material.base_color.a);
}
