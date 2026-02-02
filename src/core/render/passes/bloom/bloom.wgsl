struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct BloomParams {
    params0: vec4<f32>, // threshold, knee, texel.x, texel.y
    params1: vec4<f32>, // scatter, unused, unused, unused
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

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_input: sampler;
@group(0) @binding(2) var<uniform> bloom: BloomParams;

fn soft_threshold(color: vec3<f32>) -> vec3<f32> {
    let threshold = bloom.params0.x;
    let knee = max(bloom.params0.y, 0.0001);
    let soft = clamp((max(color, vec3<f32>(0.0)) - threshold + knee) / (2.0 * knee), vec3<f32>(0.0), vec3<f32>(1.0));
    return max(color - threshold, vec3<f32>(0.0)) + soft * soft * (knee * 0.25);
}

fn sample_box(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    sum += textureSample(t_input, s_input, uv + texel * vec2<f32>(-1.0, -1.0)).rgb;
    sum += textureSample(t_input, s_input, uv + texel * vec2<f32>(1.0, -1.0)).rgb;
    sum += textureSample(t_input, s_input, uv + texel * vec2<f32>(-1.0, 1.0)).rgb;
    sum += textureSample(t_input, s_input, uv + texel * vec2<f32>(1.0, 1.0)).rgb;
    sum += textureSample(t_input, s_input, uv).rgb;
    return sum * 0.2;
}

@fragment
fn fs_prefilter(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw;
    let color = sample_box(in.uv, texel);
    let thresholded = soft_threshold(color);
    return vec4<f32>(thresholded, 1.0);
}

@fragment
fn fs_downsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw * 2.0;
    let color = sample_box(in.uv, texel);
    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_upsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw;
    let color = sample_box(in.uv, texel);
    let scatter = bloom.params1.x;
    return vec4<f32>(color * scatter, 1.0);
}

@fragment
fn fs_combine(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_input, s_input, in.uv).rgb;
    return vec4<f32>(color, 1.0);
}
