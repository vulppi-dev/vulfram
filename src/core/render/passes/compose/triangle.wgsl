struct VOutput {
  @location(0) color: vec4<f32>,
  @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VOutput {
  var positions = array<vec2<f32>, 3>(
      vec2<f32>(0.0, 0.5),
      vec2<f32>(-0.5, -0.5),
      vec2<f32>(0.5, -0.5),
  );
  var colors = array<vec4<f32>, 3>(
      vec4<f32>(1.0, 0.0, 0.0, 1.0),
      vec4<f32>(0.0, 1.0, 0.0, 1.0),
      vec4<f32>(0.0, 0.0, 1.0, 1.0),
  );

  var output: VOutput;
  output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
  output.color = colors[vertex_index];
  return output;
}

@fragment
fn fs_main(input: VOutput) -> @location(0) vec4<f32> {
  return input.color;
}