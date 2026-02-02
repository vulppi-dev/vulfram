struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct PostParams {
    params0: vec4<f32>, // exposure, gamma, saturation, contrast
    params1: vec4<f32>, // vignette, grain, chromatic_aberration, blur
    params2: vec4<f32>, // outline_strength, outline_threshold, posterize_steps, flags
    params3: vec4<f32>, // time, sharpen, outline_width, outline_quality
    params4: vec4<f32>, // ssao_strength, ssao_power, unused, unused
    params5: vec4<f32>, // bloom_threshold, bloom_knee, bloom_intensity, bloom_scatter
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

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var<uniform> post: PostParams;
@group(0) @binding(3) var t_outline: texture_2d<f32>;
@group(0) @binding(4) var t_ssao: texture_2d<f32>;
@group(0) @binding(5) var t_bloom: texture_2d<f32>;

fn luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

fn rand(uv: vec2<f32>, time: f32) -> f32 {
    let seed = dot(uv, vec2<f32>(12.9898, 78.233)) + time * 0.01;
    return fract(sin(seed) * 43758.5453);
}

fn sample_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, uv);
}

fn sample_outline(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_outline, s_diffuse, uv);
}

fn sample_ssao(uv: vec2<f32>) -> f32 {
    return textureSample(t_ssao, s_diffuse, uv).r;
}

fn sample_bloom(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(t_bloom, s_diffuse, uv).rgb;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let exposure = post.params0.x;
    let gamma = post.params0.y;
    let saturation = post.params0.z;
    let contrast = post.params0.w;

    let vignette = post.params1.x;
    let grain = post.params1.y;
    let chroma = post.params1.z;
    let blur = post.params1.w;

    let outline_strength = post.params2.x;
    let outline_threshold = post.params2.y;
    let posterize_steps = post.params2.z;
    let flags = u32(post.params2.w + 0.5);

    let enabled = (flags & 1u) != 0u;
    let cell_shading = (flags & 2u) != 0u;
    let outline_enabled = (flags & 4u) != 0u;
    let ssao_enabled = (flags & 8u) != 0u;
    let bloom_enabled = (flags & 16u) != 0u;
    let ssao_strength = post.params4.x;
    let bloom_intensity = post.params5.z;
    let sharpen = post.params3.y;
    let outline_width = clamp(post.params3.z, 0.5, 8.0);
    let outline_quality = clamp(post.params3.w, 0.0, 1.0);
    let outline_thresh = clamp(outline_threshold, 0.0, 0.999);

    if (!enabled) {
        return sample_color(in.uv);
    }

    var color = sample_color(in.uv);

    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    let texel = 1.0 / tex_size;

    if (chroma > 0.0001) {
        let offset = (in.uv - vec2<f32>(0.5, 0.5)) * chroma * 2.0 * texel;
        let r = sample_color(in.uv + offset).r;
        let g = color.g;
        let b = sample_color(in.uv - offset).b;
        color = vec4<f32>(r, g, b, color.a);
    }

    if (blur > 0.0001) {
        let b = blur * texel;
        var sum = color.rgb * 0.4;
        sum += sample_color(in.uv + vec2<f32>(b.x, 0.0)).rgb * 0.15;
        sum += sample_color(in.uv - vec2<f32>(b.x, 0.0)).rgb * 0.15;
        sum += sample_color(in.uv + vec2<f32>(0.0, b.y)).rgb * 0.15;
        sum += sample_color(in.uv - vec2<f32>(0.0, b.y)).rgb * 0.15;
        color = vec4<f32>(sum, color.a);
    }

    if (sharpen > 0.0001) {
        let t = texel;
        let c = color.rgb;
        let n = sample_color(in.uv + vec2<f32>(t.x, 0.0)).rgb;
        let s = sample_color(in.uv - vec2<f32>(t.x, 0.0)).rgb;
        let e = sample_color(in.uv + vec2<f32>(0.0, t.y)).rgb;
        let w = sample_color(in.uv - vec2<f32>(0.0, t.y)).rgb;
        let edge = (n + s + e + w) * 0.25;
        let sharpened = mix(c, c + (c - edge), sharpen);
        color = vec4<f32>(sharpened, color.a);
    }

    var outline_mask = 0.0;
    var outline_rgb = vec3<f32>(0.0);

    if (outline_enabled && outline_strength > 0.0001) {
        let step = texel * outline_width;
        let center_outline = sample_outline(in.uv);
        let center_alpha = center_outline.a;

        var max_alpha = center_alpha;
        var max_color = center_outline.rgb;

        if (outline_quality < 0.5) {
            for (var y: i32 = -1; y <= 1; y = y + 1) {
                for (var x: i32 = -1; x <= 1; x = x + 1) {
                    if (x == 0 && y == 0) {
                        continue;
                    }
                    let offset = vec2<f32>(f32(x), f32(y)) * step;
                    let sample = sample_outline(in.uv + offset);
                    if (sample.a > max_alpha) {
                        max_alpha = sample.a;
                        max_color = sample.rgb;
                    }
                }
            }
        } else {
            for (var y: i32 = -2; y <= 2; y = y + 1) {
                for (var x: i32 = -2; x <= 2; x = x + 1) {
                    if (x == 0 && y == 0) {
                        continue;
                    }
                    let offset = vec2<f32>(f32(x), f32(y)) * step;
                    let sample = sample_outline(in.uv + offset);
                    if (sample.a > max_alpha) {
                        max_alpha = sample.a;
                        max_color = sample.rgb;
                    }
                }
            }
        }

        let outline_edge = clamp(max_alpha - center_alpha, 0.0, 1.0);
        outline_mask = smoothstep(outline_thresh, outline_thresh + 0.15, outline_edge);
        outline_rgb = max_color;
    }

    // Simple Reinhard tonemapping
    let tone_in = color.rgb * exposure;
    let tone_mapped = tone_in / (tone_in + vec3<f32>(1.0));
    color = vec4<f32>(tone_mapped, color.a);

    if (posterize_steps > 1.0) {
        let steps = max(posterize_steps, 2.0);
        let posterized = floor(color.rgb * steps) / steps;
        color = vec4<f32>(posterized, color.a);
    }

    if (cell_shading) {
        let levels = max(posterize_steps, 4.0);
        let lum = luma(color.rgb);
        let band = floor(lum * levels) / levels;
        let shaded = color.rgb * mix(0.5, 1.5, band);
        color = vec4<f32>(shaded, color.a);
    }

    if (bloom_enabled && bloom_intensity > 0.0001) {
        let bloom = sample_bloom(in.uv);
        color = vec4<f32>(color.rgb + bloom * bloom_intensity, color.a);
    }

    if (ssao_enabled && ssao_strength > 0.0001) {
        let ao = mix(1.0, sample_ssao(in.uv), clamp(ssao_strength, 0.0, 1.0));
        color = vec4<f32>(color.rgb * ao, color.a);
    }

    let lum = luma(color.rgb);
    let sat = mix(vec3<f32>(lum), color.rgb, saturation);
    let con = (sat - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5);
    let gam = pow(max(con, vec3<f32>(0.0)), vec3<f32>(1.0 / gamma));
    color = vec4<f32>(gam, color.a);

    if (vignette > 0.0001) {
        let d = distance(in.uv, vec2<f32>(0.5, 0.5));
        let v = smoothstep(0.7, 0.95, d);
        let vcol = color.rgb * mix(1.0, 1.0 - vignette, v);
        color = vec4<f32>(vcol, color.a);
    }

    if (grain > 0.0001) {
        let noise = rand(in.uv * 2048.0, post.params3.x) - 0.5;
        let gcol = color.rgb + noise * grain;
        color = vec4<f32>(gcol, color.a);
    }

    if (outline_mask > 0.0) {
        let outlined = mix(color.rgb, outline_rgb, outline_mask * outline_strength);
        color = vec4<f32>(outlined, color.a);
    }

    return vec4<f32>(color.rgb, color.a);
}
