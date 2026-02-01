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
    normal_bias: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

struct Model {
    transform: mat4x4<f32>,
    translation: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    flags: vec4<u32>, // x: flags, y: bone_offset, z: bone_count
    outline_color: vec4<f32>,
}

struct MaterialStandardParams {
    input_indices: vec4<u32>,
    inputs_offset_count: vec2<u32>,
    surface_flags: vec2<u32>,
    texture_slots: array<vec4<u32>, 2>,
    sampler_indices: array<vec4<u32>, 2>,
    tex_sources: array<vec4<u32>, 2>,
    atlas_layers: array<vec4<u32>, 2>,
    atlas_scale_bias: array<vec4<f32>, 8>,
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
@group(0) @binding(9) var<storage, read> point_light_vp: array<mat4x4<f32>>;
@group(0) @binding(10) var point_clamp_sampler: sampler;
@group(0) @binding(11) var linear_clamp_sampler: sampler;
@group(0) @binding(12) var point_repeat_sampler: sampler;
@group(0) @binding(13) var linear_repeat_sampler: sampler;
@group(0) @binding(14) var shadow_sampler: sampler_comparison;
@group(0) @binding(15) var forward_atlas: texture_2d_array<f32>;

@group(1) @binding(0) var<storage, read> models: array<Model>;
@group(1) @binding(1) var<uniform> material: MaterialStandardParams;
@group(1) @binding(2) var<storage, read> material_inputs: array<vec4<f32>>;
@group(1) @binding(3) var material_tex0: texture_2d<f32>;
@group(1) @binding(4) var material_tex1: texture_2d<f32>;
@group(1) @binding(5) var material_tex2: texture_2d<f32>;
@group(1) @binding(6) var material_tex3: texture_2d<f32>;
@group(1) @binding(7) var material_tex4: texture_2d<f32>;
@group(1) @binding(8) var material_tex5: texture_2d<f32>;
@group(1) @binding(9) var material_tex6: texture_2d<f32>;
@group(1) @binding(10) var material_tex7: texture_2d<f32>;
@group(1) @binding(11) var<storage, read> bones: array<mat4x4<f32>>;

const STANDARD_INVALID_SLOT: u32 = 0xFFFFFFFFu;
const SURFACE_MASKED: u32 = 1u;
const ALPHA_CUTOFF: f32 = 0.5;
const TEX_BASE: u32 = 0u;
const TEX_SPEC: u32 = 1u;
const TEX_NORMAL: u32 = 2u;
const TEX_TOON: u32 = 3u;
const STANDARD_FLAG_SPECULAR: u32 = 1u;
const SAMPLER_POINT_CLAMP: u32 = 0u;
const SAMPLER_LINEAR_CLAMP: u32 = 1u;
const SAMPLER_POINT_REPEAT: u32 = 2u;
const SAMPLER_LINEAR_REPEAT: u32 = 3u;
const TEX_SOURCE_STANDALONE: u32 = 0u;
const TEX_SOURCE_ATLAS: u32 = 1u;
const TEX_SOURCE_INVALID: u32 = 2u;

fn get_slot(slots: array<vec4<u32>, 2>, index: u32) -> u32 {
    let vec_index = index / 4u;
    let lane = index % 4u;
    let v = slots[vec_index];
    if (lane == 0u) { return v.x; }
    if (lane == 1u) { return v.y; }
    if (lane == 2u) { return v.z; }
    return v.w;
}

fn input_at(index: u32) -> vec4<f32> {
    return material_inputs[material.inputs_offset_count.x + index];
}

fn sample_color(tex_slot: u32, sampler_index: u32, uv: vec2<f32>) -> vec4<f32> {
    if (tex_slot == STANDARD_INVALID_SLOT) {
        return vec4<f32>(1.0);
    }
    if (tex_slot == 0u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex0, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex0, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex0, point_repeat_sampler, uv); }
        return textureSample(material_tex0, linear_repeat_sampler, uv);
    }
    if (tex_slot == 1u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex1, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex1, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex1, point_repeat_sampler, uv); }
        return textureSample(material_tex1, linear_repeat_sampler, uv);
    }
    if (tex_slot == 2u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex2, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex2, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex2, point_repeat_sampler, uv); }
        return textureSample(material_tex2, linear_repeat_sampler, uv);
    }
    if (tex_slot == 3u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex3, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex3, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex3, point_repeat_sampler, uv); }
        return textureSample(material_tex3, linear_repeat_sampler, uv);
    }
    if (tex_slot == 4u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex4, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex4, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex4, point_repeat_sampler, uv); }
        return textureSample(material_tex4, linear_repeat_sampler, uv);
    }
    if (tex_slot == 5u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex5, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex5, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex5, point_repeat_sampler, uv); }
        return textureSample(material_tex5, linear_repeat_sampler, uv);
    }
    if (tex_slot == 6u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex6, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex6, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex6, point_repeat_sampler, uv); }
        return textureSample(material_tex6, linear_repeat_sampler, uv);
    }
    if (tex_slot == 7u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex7, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex7, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex7, point_repeat_sampler, uv); }
        return textureSample(material_tex7, linear_repeat_sampler, uv);
    }
    return vec4<f32>(1.0);
}

fn sample_atlas(sampler_index: u32, uv: vec2<f32>, layer: u32) -> vec4<f32> {
    let layer_i = i32(layer);
    if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(forward_atlas, point_clamp_sampler, uv, layer_i); }
    if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(forward_atlas, linear_clamp_sampler, uv, layer_i); }
    if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(forward_atlas, point_repeat_sampler, uv, layer_i); }
    return textureSample(forward_atlas, linear_repeat_sampler, uv, layer_i);
}

fn sample_material(tex_slot: u32, sampler_index: u32, uv: vec2<f32>) -> vec4<f32> {
    if (tex_slot == STANDARD_INVALID_SLOT) {
        return vec4<f32>(1.0);
    }

    let source = get_slot(material.tex_sources, tex_slot);
    if (source == TEX_SOURCE_ATLAS) {
        let scale_bias = material.atlas_scale_bias[tex_slot];
        let atlas_uv = uv * scale_bias.xy + scale_bias.zw;
        let layer = get_slot(material.atlas_layers, tex_slot);
        return sample_atlas(sampler_index, atlas_uv, layer);
    }
    if (source == TEX_SOURCE_INVALID) {
        return vec4<f32>(1.0);
    }

    return sample_color(tex_slot, sampler_index, uv);
}

fn diffuse_term(ndotl: f32, toon_slot: u32, toon_sampler: u32) -> vec3<f32> {
    if (toon_slot == STANDARD_INVALID_SLOT) {
        return vec3<f32>(ndotl);
    }
    let ramp = sample_material(toon_slot, toon_sampler, vec2<f32>(ndotl, 0.5));
    return ramp.rgb;
}

fn apply_normal_map(
    normal: vec3<f32>,
    world_pos: vec3<f32>,
    uv: vec2<f32>,
    normal_slot: u32,
    normal_sampler: u32,
) -> vec3<f32> {
    if (normal_slot == STANDARD_INVALID_SLOT) {
        return normalize(normal);
    }
    let n = normalize(normal);
    let dp1 = dpdx(world_pos);
    let dp2 = dpdy(world_pos);
    let duv1 = dpdx(uv);
    let duv2 = dpdy(uv);
    let t = normalize(dp1 * duv2.y - dp2 * duv1.y);
    let b = normalize(-dp1 * duv2.x + dp2 * duv1.x);
    let map = sample_material(normal_slot, normal_sampler, uv).xyz * 2.0 - 1.0;
    let tbn = mat3x3<f32>(t, b, n);
    return normalize(tbn * map);
}

// -----------------------------------------------------------------------------
// Vertex I/O
// -----------------------------------------------------------------------------

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(4) uv0: vec2<f32>,
    @location(3) color0: vec4<f32>,
    @location(6) joints: vec4<u32>,
    @location(7) weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) color0: vec4<f32>,
    @location(4) @interpolate(flat) instance_id: u32,
}

// -----------------------------------------------------------------------------
// Shadow sampling helpers (ported from legacy forward shader)
// -----------------------------------------------------------------------------

fn compute_table_id(light_base: u32, grid_x: u32, grid_y: u32, grid_size_u: u32) -> u32 {
    let grid_area = grid_size_u * grid_size_u;
    let linear_id = light_base * grid_area + grid_y * grid_size_u + grid_x;
    return linear_id % shadow_params.table_capacity;
}

// -----------------------------------------------------------------------------
// Skinning helpers
// -----------------------------------------------------------------------------

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

fn skin_normal(
    normal: vec3<f32>,
    joints: vec4<u32>,
    weights: vec4<f32>,
    bone_offset: u32,
    bone_count: u32,
) -> vec3<f32> {
    let m0 = bone_at(joints.x, bone_offset, bone_count);
    let m1 = bone_at(joints.y, bone_offset, bone_count);
    let m2 = bone_at(joints.z, bone_offset, bone_count);
    let m3 = bone_at(joints.w, bone_offset, bone_count);
    let n0 = (mat3x3<f32>(m0[0].xyz, m0[1].xyz, m0[2].xyz) * normal) * weights.x;
    let n1 = (mat3x3<f32>(m1[0].xyz, m1[1].xyz, m1[2].xyz) * normal) * weights.y;
    let n2 = (mat3x3<f32>(m2[0].xyz, m2[1].xyz, m2[2].xyz) * normal) * weights.z;
    let n3 = (mat3x3<f32>(m3[0].xyz, m3[1].xyz, m3[2].xyz) * normal) * weights.w;
    return normalize(n0 + n1 + n2 + n3);
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
    let ndc_in_bounds = light_uv.x >= 0.0
        && light_uv.x <= 1.0
        && light_uv.y >= 0.0
        && light_uv.y <= 1.0
        && light_depth >= 0.0
        && light_depth <= 1.0;

    let grid_size_f = shadow_params.virtual_grid_size;
    let grid_size_u = u32(grid_size_f);
    let grid_x = u32(clamp(light_uv.x * grid_size_f, 0.0, grid_size_f - 1.0));
    let grid_y = u32(clamp(light_uv.y * grid_size_f, 0.0, grid_size_f - 1.0));

    let table_id = compute_table_id(light_base, grid_x, grid_y, grid_size_u);
    let page = shadow_page_table[table_id];
    let page_has_data = !(page.scale_offset.x == 0.0 && page.scale_offset.y == 0.0);

    let page_origin = vec2<f32>(f32(grid_x), f32(grid_y)) / grid_size_f;
    let page_uv = (light_uv - page_origin) * grid_size_f;
    let page_in_bounds = page_uv.x >= 0.0
        && page_uv.x <= 1.0
        && page_uv.y >= 0.0
        && page_uv.y <= 1.0;
    let shadow_valid = ndc_in_bounds && page_has_data && page_in_bounds;

    let atlas_uv_center = (page_uv * page.scale_offset.xy) + page.scale_offset.zw;
    let bias = max(bias_min, bias_slope * (1.0 - ndotl));

    let dim = textureDimensions(shadow_atlas);
    let atlas_texel = 1.0 / vec2<f32>(f32(dim.x), f32(dim.y));
    let tile_min = page.scale_offset.zw;
    let tile_max = page.scale_offset.zw + page.scale_offset.xy;
    let guard = atlas_texel * 1.5;
    let uv_min = tile_min + guard;
    let uv_max = tile_max - guard;

    var shadow = 1.0;
    if (shadow_params.pcf_range == 0) {
        let uv = clamp(atlas_uv_center, uv_min, uv_max);
        shadow = textureSampleCompare(
            shadow_atlas,
            shadow_sampler,
            uv,
            i32(page.layer_index),
            saturate(light_depth + bias) // Reverse Z: add bias (larger Z is closer)
        );
    } else {
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
                    saturate(light_depth + bias) // Reverse Z: add bias
                );
                samples += 1.0;
            }
        }
        shadow = sum / samples;
    }

    return select(1.0, shadow, shadow_valid);
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
    let in_bounds = ndc.x >= -1.0
        && ndc.x <= 1.0
        && ndc.y >= -1.0
        && ndc.y <= 1.0
        && ndc.z >= 0.0
        && ndc.z <= 1.0;
    let light_base = shadow_idx * 6u + face;
    let shadow = sample_shadow_page_at(light_base, ndc, ndotl, shadow_params.point_bias_min, shadow_params.point_bias_slope);
    return select(1.0, shadow, in_bounds);
}

fn point_shadow_factor(light: Light, world_pos: vec3<f32>, ndotl: f32) -> f32 {
    let v = world_pos - light.position.xyz;
    let dist = length(v);
    let dist_valid = dist > 1e-5;
    let inv_dist = select(0.0, 1.0 / dist, dist_valid);
    let dir = v * inv_dist;
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
    let shadow = min(s0, s1);
    return select(1.0, shadow, dist_valid);
}

fn get_shadow_factor(
    light: Light,
    world_pos: vec3<f32>,
    shadow_normal: vec3<f32>,
    ndotl: f32,
    instance_id: u32
) -> f32 {
    let model = models[instance_id];
    let model_receive_shadow = (model.flags.x & 1u) != 0u;
    let light_cast_shadow = (light.kind_flags.y & 1u) != 0u;
    let shadow_enabled = model_receive_shadow && light_cast_shadow && ndotl > 0.0;
    let offset_pos = world_pos + shadow_normal * shadow_params.normal_bias;
    let light_clip = light.view_projection * vec4<f32>(offset_pos, 1.0);
    let light_ndc = light_clip.xyz / light_clip.w;
    let shadow_idx = light.shadow_index;
    let light_base = shadow_idx * 6u + 0u;
    let directional_shadow =
        sample_shadow_page_at(light_base, light_ndc, ndotl, shadow_params.bias_min, shadow_params.bias_slope);
    let point_shadow = point_shadow_factor(light, offset_pos, ndotl);
    let is_point = light.kind_flags.x == 1u;
    let shadow = select(directional_shadow, point_shadow, is_point);
    return select(1.0, shadow, shadow_enabled);
}

// -----------------------------------------------------------------------------
// Lighting (Standard: Lambert + Phong)
// -----------------------------------------------------------------------------

fn calculate_directional_light(
    light: Light,
    normal: vec3<f32>,
    shadow_normal: vec3<f32>,
    world_pos: vec3<f32>,
    toon_slot: u32,
    toon_sampler: u32,
    instance_id: u32,
) -> vec3<f32> {
    let l = normalize(-light.direction.xyz);
    let ndotl = max(dot(normal, l), 0.0);
    let ndotl_shadow = max(dot(shadow_normal, l), 0.0);
    let shadow = get_shadow_factor(light, world_pos, shadow_normal, ndotl_shadow, instance_id);
    let diffuse = diffuse_term(ndotl, toon_slot, toon_sampler);
    let lighting = light.color.rgb * light.intensity_range.x * diffuse * shadow;
    return select(vec3<f32>(0.0), lighting, ndotl > 0.0);
}

fn calculate_ambient_light(light: Light) -> vec3<f32> {
    return light.color.rgb * light.intensity_range.x;
}

fn calculate_hemisphere_light(light: Light, normal: vec3<f32>) -> vec3<f32> {
    let up = normalize(light.direction.xyz);
    let w = dot(normal, up) * 0.5 + 0.5;
    return mix(light.ground_color.rgb, light.color.rgb, w) * light.intensity_range.x;
}

fn calculate_spot_light(
    light: Light,
    normal: vec3<f32>,
    shadow_normal: vec3<f32>,
    world_pos: vec3<f32>,
    toon_slot: u32,
    toon_sampler: u32,
    instance_id: u32,
) -> vec3<f32> {
    let light_to_pos = world_pos - light.position.xyz;
    let dist = length(light_to_pos);
    let range = light.intensity_range.y;
    let dist_valid = dist > 1e-5;
    let inv_dist = select(0.0, 1.0 / dist, dist_valid);
    let l = -light_to_pos * inv_dist;
    let safe_range = max(range, 1e-5);
    let in_range = dist <= range;
    let attenuation = pow(clamp(1.0 - dist / safe_range, 0.0, 1.0), 2.0);
    let theta = dot(l, normalize(-light.direction.xyz));
    let inner = cos(light.spot_inner_outer.x);
    let outer = cos(light.spot_inner_outer.y);
    let epsilon = max(inner - outer, 1e-5);
    let spot_intensity = clamp((theta - outer) / epsilon, 0.0, 1.0);
    let ndotl = max(dot(normal, l), 0.0);
    let ndotl_shadow = max(dot(shadow_normal, l), 0.0);
    let shadow = get_shadow_factor(light, world_pos, shadow_normal, ndotl_shadow, instance_id);
    let diffuse = diffuse_term(ndotl, toon_slot, toon_sampler);
    let lighting = light.color.rgb * light.intensity_range.x * diffuse * attenuation * spot_intensity * shadow;
    let enabled = dist_valid && in_range && ndotl > 0.0;
    return select(vec3<f32>(0.0), lighting, enabled);
}

fn calculate_point_light(
    light: Light,
    normal: vec3<f32>,
    shadow_normal: vec3<f32>,
    world_pos: vec3<f32>,
    toon_slot: u32,
    toon_sampler: u32,
    instance_id: u32,
) -> vec3<f32> {
    let light_to_pos = world_pos - light.position.xyz;
    let dist = length(light_to_pos);
    let range = light.intensity_range.y;
    let dist_valid = dist > 1e-5;
    let inv_dist = select(0.0, 1.0 / dist, dist_valid);
    let l = -light_to_pos * inv_dist;
    let safe_range = max(range, 1e-5);
    let in_range = dist <= range;
    let attenuation = pow(clamp(1.0 - dist / safe_range, 0.0, 1.0), 2.0);
    let ndotl = max(dot(normal, l), 0.0);
    let ndotl_shadow = max(dot(shadow_normal, l), 0.0);
    let shadow = get_shadow_factor(light, world_pos, shadow_normal, ndotl_shadow, instance_id);
    let diffuse = diffuse_term(ndotl, toon_slot, toon_sampler);
    let lighting = light.color.rgb * light.intensity_range.x * diffuse * attenuation * shadow;
    let enabled = dist_valid && in_range && ndotl > 0.0;
    return select(vec3<f32>(0.0), lighting, enabled);
}

// -----------------------------------------------------------------------------
// Vertex
// -----------------------------------------------------------------------------

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_id: u32) -> VertexOutput {
    var out: VertexOutput;
    let model = models[instance_id];
    let bone_offset = model.flags.y;
    let bone_count = model.flags.z;
    var local_pos = in.position;
    var local_normal = in.normal;
    if (bone_count > 0u) {
        local_pos = skin_position(in.position, in.joints, in.weights, bone_offset, bone_count);
        local_normal = skin_normal(in.normal, in.joints, in.weights, bone_offset, bone_count);
    }
    let world_pos = model.transform * vec4<f32>(local_pos, 1.0);
    out.clip_position = camera.view_projection * world_pos;
    out.world_position = world_pos.xyz;
    out.normal = (model.transform * vec4<f32>(local_normal, 0.0)).xyz;
    out.uv0 = in.uv0;
    out.color0 = in.color0;
    out.instance_id = instance_id;
    return out;
}

// -----------------------------------------------------------------------------
// Fragment
// -----------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = input_at(material.input_indices.x);
    let spec_enabled = (material.surface_flags.y & STANDARD_FLAG_SPECULAR) != 0u;

    let base_tex_slot = get_slot(material.texture_slots, TEX_BASE);
    let base_sampler = get_slot(material.sampler_indices, TEX_BASE);
    let base_tex = sample_material(base_tex_slot, base_sampler, in.uv0);
    let toon_slot = get_slot(material.texture_slots, TEX_TOON);
    let toon_sampler = get_slot(material.sampler_indices, TEX_TOON);

    var color = base_color.rgb * base_tex.rgb * in.color0.rgb;
    let alpha = base_color.a * base_tex.a;

    let cam = light_params.camera_index;
    let base = cam * light_params.max_lights_per_camera;
    let count = min(visible_counts[cam], light_params.max_lights_per_camera);

    if (count > 0u) {
        let normal_slot = get_slot(material.texture_slots, TEX_NORMAL);
        let normal_sampler = get_slot(material.sampler_indices, TEX_NORMAL);
        let n_geom = normalize(in.normal);
        let n = apply_normal_map(in.normal, in.world_position, in.uv0, normal_slot, normal_sampler);
        var spec_color_final = vec3<f32>(0.0);
        var spec_power = 0.0;
        var view_dir = vec3<f32>(0.0);
        if (spec_enabled) {
            let spec_color = input_at(material.input_indices.y);
            spec_power = input_at(material.input_indices.z).x;
            let spec_slot = get_slot(material.texture_slots, TEX_SPEC);
            let spec_sampler = get_slot(material.sampler_indices, TEX_SPEC);
            let spec_tex = sample_material(spec_slot, spec_sampler, in.uv0);
            spec_color_final = spec_color.rgb * spec_tex.rgb;
            view_dir = normalize(camera.position.xyz - in.world_position);
        }
        var lighting = vec3<f32>(0.0);
        var specular = vec3<f32>(0.0);
        for (var i = 0u; i < count; i++) {
            let idx = visible_indices[base + i];
            let light = lights[idx];
            let kind = light.kind_flags.x;
            if (kind == 0u) {
                let dir_light = calculate_directional_light(
                    light,
                    n,
                    n_geom,
                    in.world_position,
                    toon_slot,
                    toon_sampler,
                    in.instance_id
                );
                lighting += dir_light;
                if (spec_enabled) {
                    let light_dir = normalize(-light.direction.xyz);
                    let reflect_dir = reflect(-light_dir, n);
                    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), spec_power);
                    specular += spec_color_final * spec * light.intensity_range.x;
                }
            } else if (kind == 1u) {
                let point_light = calculate_point_light(
                    light,
                    n,
                    n_geom,
                    in.world_position,
                    toon_slot,
                    toon_sampler,
                    in.instance_id
                );
                lighting += point_light;
                if (spec_enabled) {
                    let light_dir = normalize(light.position.xyz - in.world_position);
                    let reflect_dir = reflect(-light_dir, n);
                    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), spec_power);
                    specular += spec_color_final * spec * light.intensity_range.x;
                }
            } else if (kind == 2u) {
                let spot_light = calculate_spot_light(
                    light,
                    n,
                    n_geom,
                    in.world_position,
                    toon_slot,
                    toon_sampler,
                    in.instance_id
                );
                lighting += spot_light;
                if (spec_enabled) {
                    let light_dir = normalize(light.position.xyz - in.world_position);
                    let reflect_dir = reflect(-light_dir, n);
                    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), spec_power);
                    specular += spec_color_final * spec * light.intensity_range.x;
                }
            } else if (kind == 3u) {
                lighting += calculate_ambient_light(light);
            } else if (kind == 4u) {
                lighting += calculate_hemisphere_light(light, n);
            }
        }
        color *= (lighting + vec3<f32>(0.001));
        color += specular;
    } else {
        color *= vec3<f32>(0.001);
    }

    if (material.surface_flags.x == SURFACE_MASKED && alpha < ALPHA_CUTOFF) {
        discard;
    }
    return vec4<f32>(color, alpha);
}
