struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct SsaoBlurParams {
    params0: vec4<f32>, // texel.x, texel.y, depth_threshold, radius
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

@group(0) @binding(0) var t_ssao: texture_2d<f32>;
@group(0) @binding(1) var t_depth: texture_depth_multisampled_2d;
@group(0) @binding(2) var<uniform> blur: SsaoBlurParams;

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

fn ssao_at(uv: vec2<f32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(t_ssao));
    let coord = clamp(uv * dims, vec2<f32>(0.0), dims - vec2<f32>(1.0));
    return textureLoad(t_ssao, vec2<i32>(coord), 0).r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = blur.params0.xy;
    let depth_threshold = blur.params0.z;
    let radius = i32(blur.params0.w + 0.5);

    let center_depth = depth_at(in.uv);
    var sum = 0.0;
    var weight_sum = 0.0;

    for (var y: i32 = -radius; y <= radius; y = y + 1) {
        for (var x: i32 = -radius; x <= radius; x = x + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            let uv = in.uv + offset;
            let sample = ssao_at(uv);
            let sample_depth = depth_at(uv);
            let depth_diff = abs(sample_depth - center_depth);
            let depth_weight = 1.0 - clamp(depth_diff / depth_threshold, 0.0, 1.0);
            let dist = length(vec2<f32>(f32(x), f32(y)));
            let blur_weight = exp(-dist * dist * 0.5);
            let weight = depth_weight * blur_weight;
            sum += sample * weight;
            weight_sum += weight;
        }
    }

    let result = sum / max(weight_sum, 0.0001);
    return vec4<f32>(result, result, result, 1.0);
}
