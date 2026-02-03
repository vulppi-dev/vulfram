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
@group(0) @binding(2) var<storage, read> bloom: BloomParams;

fn soft_threshold(color: vec3<f32>) -> vec3<f32> {
    let threshold = bloom.params0.x;
    let knee = max(bloom.params0.y, 0.0001);
    let soft = clamp((max(color, vec3<f32>(0.0)) - threshold + knee) / (2.0 * knee), vec3<f32>(0.0), vec3<f32>(1.0));
    return max(color - threshold, vec3<f32>(0.0)) + soft * soft * (knee * 0.25);
}

fn sample_tent(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    // 9-tap tent filter (1-2-1 kernel), normalized by 16.
    let tl = textureSample(t_input, s_input, uv + texel * vec2<f32>(-1.0, -1.0)).rgb;
    let tc = textureSample(t_input, s_input, uv + texel * vec2<f32>(0.0, -1.0)).rgb;
    let tr = textureSample(t_input, s_input, uv + texel * vec2<f32>(1.0, -1.0)).rgb;
    let ml = textureSample(t_input, s_input, uv + texel * vec2<f32>(-1.0, 0.0)).rgb;
    let mc = textureSample(t_input, s_input, uv).rgb;
    let mr = textureSample(t_input, s_input, uv + texel * vec2<f32>(1.0, 0.0)).rgb;
    let bl = textureSample(t_input, s_input, uv + texel * vec2<f32>(-1.0, 1.0)).rgb;
    let bc = textureSample(t_input, s_input, uv + texel * vec2<f32>(0.0, 1.0)).rgb;
    let br = textureSample(t_input, s_input, uv + texel * vec2<f32>(1.0, 1.0)).rgb;

    let row0 = tl + tc * 2.0 + tr;
    let row1 = ml * 2.0 + mc * 4.0 + mr * 2.0;
    let row2 = bl + bc * 2.0 + br;
    return (row0 + row1 + row2) * (1.0 / 16.0);
}

fn sample_gaussian_h(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    let w0 = 0.227027;
    let w1 = 0.1945946;
    let w2 = 0.1216216;
    let w3 = 0.054054;
    let c0 = textureSample(t_input, s_input, uv).rgb * w0;
    let c1 = textureSample(t_input, s_input, uv + vec2<f32>(texel.x, 0.0)).rgb * w1;
    let c2 = textureSample(t_input, s_input, uv - vec2<f32>(texel.x, 0.0)).rgb * w1;
    let c3 = textureSample(t_input, s_input, uv + vec2<f32>(texel.x * 2.0, 0.0)).rgb * w2;
    let c4 = textureSample(t_input, s_input, uv - vec2<f32>(texel.x * 2.0, 0.0)).rgb * w2;
    let c5 = textureSample(t_input, s_input, uv + vec2<f32>(texel.x * 3.0, 0.0)).rgb * w3;
    let c6 = textureSample(t_input, s_input, uv - vec2<f32>(texel.x * 3.0, 0.0)).rgb * w3;
    return c0 + c1 + c2 + c3 + c4 + c5 + c6;
}

fn sample_gaussian_h_thresholded(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    let w0 = 0.227027;
    let w1 = 0.1945946;
    let w2 = 0.1216216;
    let w3 = 0.054054;
    let c0 = soft_threshold(textureSample(t_input, s_input, uv).rgb) * w0;
    let c1 = soft_threshold(textureSample(t_input, s_input, uv + vec2<f32>(texel.x, 0.0)).rgb) * w1;
    let c2 = soft_threshold(textureSample(t_input, s_input, uv - vec2<f32>(texel.x, 0.0)).rgb) * w1;
    let c3 = soft_threshold(textureSample(t_input, s_input, uv + vec2<f32>(texel.x * 2.0, 0.0)).rgb) * w2;
    let c4 = soft_threshold(textureSample(t_input, s_input, uv - vec2<f32>(texel.x * 2.0, 0.0)).rgb) * w2;
    let c5 = soft_threshold(textureSample(t_input, s_input, uv + vec2<f32>(texel.x * 3.0, 0.0)).rgb) * w3;
    let c6 = soft_threshold(textureSample(t_input, s_input, uv - vec2<f32>(texel.x * 3.0, 0.0)).rgb) * w3;
    return c0 + c1 + c2 + c3 + c4 + c5 + c6;
}

fn sample_gaussian_v(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    let w0 = 0.227027;
    let w1 = 0.1945946;
    let w2 = 0.1216216;
    let w3 = 0.054054;
    let c0 = textureSample(t_input, s_input, uv).rgb * w0;
    let c1 = textureSample(t_input, s_input, uv + vec2<f32>(0.0, texel.y)).rgb * w1;
    let c2 = textureSample(t_input, s_input, uv - vec2<f32>(0.0, texel.y)).rgb * w1;
    let c3 = textureSample(t_input, s_input, uv + vec2<f32>(0.0, texel.y * 2.0)).rgb * w2;
    let c4 = textureSample(t_input, s_input, uv - vec2<f32>(0.0, texel.y * 2.0)).rgb * w2;
    let c5 = textureSample(t_input, s_input, uv + vec2<f32>(0.0, texel.y * 3.0)).rgb * w3;
    let c6 = textureSample(t_input, s_input, uv - vec2<f32>(0.0, texel.y * 3.0)).rgb * w3;
    return c0 + c1 + c2 + c3 + c4 + c5 + c6;
}

@fragment
fn fs_prefilter_h(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw;
    let color = sample_gaussian_h_thresholded(in.uv, texel);
    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_prefilter_v(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw;
    let color = sample_gaussian_v(in.uv, texel);
    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_downsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw * 2.0;
    let color = sample_tent(in.uv, texel);
    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_upsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = bloom.params0.zw;
    let color = sample_tent(in.uv, texel);
    let scatter = bloom.params1.x;
    return vec4<f32>(color * scatter, 1.0);
}

@fragment
fn fs_combine(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_input, s_input, in.uv).rgb;
    return vec4<f32>(color, 1.0);
}
