// Blit shader - fullscreen quad for compositing render targets to surface

@group(0) @binding(0) var t_texture: texture_2d<f32>;
@group(0) @binding(1) var t_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Vertex shader - generates fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Generate fullscreen triangle
    // Vertex 0: (-1, -1) -> (0, 1) UV
    // Vertex 1: (3, -1) -> (2, 1) UV
    // Vertex 2: (-1, 3) -> (0, -1) UV
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;
    
    output.position = vec4<f32>(x, -y, 0.0, 1.0);
    output.tex_coords = vec2<f32>(x * 0.5 + 0.5, y * 0.5 + 0.5);
    
    return output;
}

// Fragment shader - samples texture
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, t_sampler, input.tex_coords);
}
