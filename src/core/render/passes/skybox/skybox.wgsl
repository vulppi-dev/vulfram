struct SkyboxParams {
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    intensity: vec4<f32>,
    ground_color: vec4<f32>,
    horizon_color: vec4<f32>,
    sky_color: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_sky: SkyboxParams;
@group(0) @binding(1) var t_sky: texture_2d<f32>;
@group(0) @binding(2) var s_sky: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    let pos = positions[idx];
    var out: VsOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

fn rotate_y(dir: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec3<f32>(c * dir.x + s * dir.z, dir.y, -s * dir.x + c * dir.z);
}

fn equirect_uv(dir: vec3<f32>) -> vec2<f32> {
    let n = normalize(dir);
    let u = atan2(n.z, n.x) * 0.15915494 + 0.5; // 1 / (2 * PI)
    let v = 1.0 - (asin(clamp(n.y, -1.0, 1.0)) * 0.318309886 + 0.5); // 1 / PI
    return vec2<f32>(fract(u), clamp(v, 0.0, 1.0));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let ndc = vec4<f32>(in.uv * 2.0 - 1.0, 1.0, 1.0);
    let world = u_sky.inv_view_proj * ndc;
    let world_pos = world.xyz / world.w;
    var dir = normalize(world_pos - u_sky.camera_pos.xyz);
    dir = rotate_y(dir, u_sky.params.x);

    let mode = u_sky.params.y;
    let intensity = u_sky.intensity.x;

    if (mode < 1.5) {
        let t = clamp(in.uv.y, 0.0, 1.0);
        let ground = u_sky.ground_color.xyz;
        let horizon = u_sky.horizon_color.xyz;
        let sky = u_sky.sky_color.xyz;

        let bottom_t = clamp(t * 2.0, 0.0, 1.0);
        let top_t = clamp((t - 0.5) * 2.0, 0.0, 1.0);
        let low = mix(ground, horizon, bottom_t);
        let high = mix(horizon, sky, top_t);
        let split = smoothstep(0.45, 0.55, t);
        let color = mix(low, high, split);
        let final_color = color * intensity;
        return vec4<f32>(final_color, 1.0);
    }

    let uv = equirect_uv(dir);
    let texel = textureSample(t_sky, s_sky, uv).rgb;
    return vec4<f32>(texel * intensity, 1.0);
}
