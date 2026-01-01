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
    flags: vec4<u32>, // x: flags (bit 0: receive_shadow)
}

struct LightDrawParams {
    camera_index: u32,
    max_lights_per_camera: u32,
};

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    intensity_range: vec2<f32>,
    spot_inner_outer: vec2<f32>,
    kind_flags: vec2<u32>,
    _padding: vec2<u32>,
}

struct ShadowPageEntry {
    scale_offset: vec4<f32>,
    layer_index: u32,
    _padding: vec3<u32>,
}

struct ShadowParams {
    virtual_grid_size: f32,
    pcf_range: i32,
    _padding: vec2<f32>,
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
    let world_pos = model.transform * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_projection * world_pos;
    out.world_position = world_pos.xyz;
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
    let n = normalize(in.normal);
    var color = in.color0.rgb;

    let cam = light_params.camera_index;
    let base = cam * light_params.max_lights_per_camera;
    let count = min(visible_counts[cam], light_params.max_lights_per_camera);
    
    if (count > 0u) {
        var lighting = vec3<f32>(0.0);
        for (var i = 0u; i < count; i++) {
            let idx = visible_indices[base + i];
            let light = lights[idx];
            let l = normalize(-light.direction.xyz);
            let ndotl = max(dot(n, l), 0.0);
            
            var shadow = 1.0;
            
            let model_receive_shadow = (model.flags.x & 1u) != 0u;
            let light_cast_shadow = (light.kind_flags.y & 1u) != 0u;

            if (model_receive_shadow && light_cast_shadow) {
                // --- Lógica VSM ---
                let light_clip = light.view_projection * vec4<f32>(in.world_position, 1.0);
                let light_ndc = light_clip.xyz / light_clip.w;
                
                // WGPU NDC to UV: X [-1, 1] -> [0, 1], Y [1, -1] -> [0, 1]
                let light_uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, light_ndc.y * -0.5 + 0.5);
                // Grid lookup should use the same NDC orientation used by the shadow page selection.
                let light_grid_uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, light_ndc.y * 0.5 + 0.5);
                let light_depth = light_ndc.z;

                if (light_uv.x >= 0.0 && light_uv.x <= 1.0 && light_uv.y >= 0.0 && light_uv.y <= 1.0 && light_depth >= 0.0 && light_depth <= 1.0) {
                    let virtual_grid_size = shadow_params.virtual_grid_size;
                    let grid_x = u32(clamp(light_grid_uv.x * virtual_grid_size, 0.0, virtual_grid_size - 1.0));
                    let grid_y = u32(clamp(light_grid_uv.y * virtual_grid_size, 0.0, virtual_grid_size - 1.0));
                    
                    let table_id = (idx + grid_y + grid_x) % 1024u;
                    let page = shadow_page_table[table_id];

                    if (any(page.scale_offset != vec4<f32>(0.0))) {
                        let grid_y_flipped = (virtual_grid_size - 1.0) - f32(grid_y);
                        let page_origin = vec2<f32>(f32(grid_x), grid_y_flipped) / virtual_grid_size;
                        let page_uv = clamp((light_uv - page_origin) * virtual_grid_size, vec2<f32>(0.0), vec2<f32>(1.0));
                        let atlas_uv = (page_uv * page.scale_offset.xy) + page.scale_offset.zw;
                        // Bias adaptativo simples para reduzir acne
                        let bias = max(0.002, 0.02 * (1.0 - ndotl));
                        let dim = textureDimensions(shadow_atlas);
                        let texel = 1.0 / vec2<f32>(f32(dim.x), f32(dim.y));
                        var sum = 0.0;
                        let range = shadow_params.pcf_range;
                        var samples = 0.0;
                        for (var oy = -range; oy <= range; oy = oy + 1) {
                            for (var ox = -range; ox <= range; ox = ox + 1) {
                                let offset = vec2<f32>(f32(ox), f32(oy)) * texel;
                                sum += textureSampleCompare(
                                    shadow_atlas,
                                    shadow_sampler,
                                    atlas_uv + offset,
                                    i32(page.layer_index),
                                    light_depth - bias
                                );
                                samples += 1.0;
                            }
                        }
                        shadow = sum / samples;
                    }
                }
            }

            let intensity = light.intensity_range.x;
            lighting += (light.color.rgb * intensity * ndotl * shadow + vec3<f32>(0.001));
        }
        color *= lighting;
    } else {
        color *= vec3<f32>(0.001);
    }

    return vec4<f32>(color, 1.0);
}
