struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct SsaoParams {
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    params0: vec4<f32>, // radius, bias, power, unused
    params1: vec4<f32>, // texel.x, texel.y, unused, frame
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((i32(vertex_index) << 1) & 2) * 2.0 - 1.0;
    let y = f32(i32(vertex_index) & 2) * -2.0 + 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@group(0) @binding(0) var t_depth: texture_depth_multisampled_2d;
@group(0) @binding(1) var<uniform> ssao: SsaoParams;

fn rand(v: vec2<f32>, seed: f32) -> f32 {
    let n = dot(v, vec2<f32>(12.9898, 78.233)) + seed * 0.01;
    return fract(sin(n) * 43758.5453);
}

fn rand2(v: vec2<f32>, seed: f32) -> vec2<f32> {
    return vec2<f32>(rand(v, seed), rand(v + vec2<f32>(5.2, 1.3), seed));
}

fn depth_at(uv: vec2<f32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(t_depth));
    let coord = clamp(uv * dims, vec2<f32>(0.0), dims - vec2<f32>(1.0));
    let sample_count = i32(textureNumSamples(t_depth));
    var sum = 0.0;
    for (var i: i32 = 0; i < sample_count; i = i + 1) {
        sum += textureLoad(t_depth, vec2<i32>(coord), i);
    }
    return sum / f32(sample_count);
}

fn view_pos_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let view = ssao.inv_proj * ndc;
    return view.xyz / view.w;
}

const KERNEL: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(0.15, 0.05, 0.02),
    vec3<f32>(0.35, 0.15, 0.10),
    vec3<f32>(-0.25, 0.10, 0.05),
    vec3<f32>(0.05, -0.30, 0.10),
    vec3<f32>(-0.15, -0.20, 0.20),
    vec3<f32>(0.20, 0.25, 0.30),
    vec3<f32>(-0.35, 0.05, 0.25),
    vec3<f32>(0.10, -0.35, 0.15)
);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = ssao.params0.x;
    let bias = ssao.params0.y;
    let power = ssao.params0.z;
    let texel = ssao.params1.xy;
    let seed = ssao.params1.w;

    let depth = depth_at(in.uv);
    if (depth <= 0.0) {
        return vec4<f32>(1.0);
    }

    let pos = view_pos_from_depth(in.uv, depth);
    let pos_dx = view_pos_from_depth(in.uv + vec2<f32>(texel.x, 0.0), depth_at(in.uv + vec2<f32>(texel.x, 0.0)));
    let pos_dy = view_pos_from_depth(in.uv + vec2<f32>(0.0, texel.y), depth_at(in.uv + vec2<f32>(0.0, texel.y)));

    var normal = normalize(cross(pos_dx - pos, pos_dy - pos));
    if (all(normal == vec3<f32>(0.0))) {
        normal = vec3<f32>(0.0, 0.0, 1.0);
    }

    let r = rand2(in.uv * 2048.0, seed) * 2.0 - 1.0;
    var tangent = normalize(r.xyx - normal * dot(r.xyx, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var occlusion = 0.0;
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        let sample_dir = tbn * KERNEL[i];
        let sample_pos = pos + sample_dir * radius;
        let clip = ssao.proj * vec4<f32>(sample_pos, 1.0);
        let ndc = clip.xyz / clip.w;
        let sample_uv = ndc.xy * 0.5 + 0.5;

        if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0) {
            continue;
        }

        let sample_depth = depth_at(sample_uv);
        let sample_view = view_pos_from_depth(sample_uv, sample_depth);
        let range = smoothstep(0.0, 1.0, radius / max(0.0001, abs(pos.z - sample_view.z)));
        let blocked = select(0.0, 1.0, sample_view.z > sample_pos.z + bias);
        occlusion = occlusion + blocked * range;
    }

    occlusion = 1.0 - (occlusion / 8.0);
    occlusion = pow(clamp(occlusion, 0.0, 1.0), power);

    return vec4<f32>(occlusion, occlusion, occlusion, 1.0);
}
