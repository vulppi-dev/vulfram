use crate::core::render::RenderState;

pub fn pass_forward(
    render_state: &mut RenderState,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
) {
}

// let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//     label: None,
//     source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/triangle.wgsl"))),
// });

// let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//     label: None,
//     bind_group_layouts: &[],
//     push_constant_ranges: &[],
// });

// let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//     label: None,
//     layout: Some(&pipeline_layout),
//     vertex: wgpu::VertexState {
//         module: &shader,
//         entry_point: Some("vs_main"),
//         buffers: &[],
//         compilation_options: wgpu::PipelineCompilationOptions::default(),
//     },
//     fragment: Some(wgpu::FragmentState {
//         module: &shader,
//         entry_point: Some("fs_main"),
//         targets: &[Some(wgpu::ColorTargetState {
//             format: window_state.config.format,
//             blend: None,
//             write_mask: wgpu::ColorWrites::ALL,
//         })],
//         compilation_options: wgpu::PipelineCompilationOptions::default(),
//     }),
//     primitive: wgpu::PrimitiveState::default(),
//     depth_stencil: None,
//     multisample: wgpu::MultisampleState::default(),
//     multiview: None,
//     cache: None,
// });

// {
//     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//         label: None,
//         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//             view: &view,
//             resolve_target: None,
//             ops: wgpu::Operations {
//                 load: wgpu::LoadOp::Clear(wgpu::Color {
//                     r: 0.1,
//                     g: 0.2,
//                     b: 0.3,
//                     a: 1.0,
//                 }),
//                 store: wgpu::StoreOp::Store,
//             },
//             depth_slice: None,
//         })],
//         depth_stencil_attachment: None,
//         ..Default::default()
//     });

//     render_pass.set_pipeline(&render_pipeline);
//     render_pass.draw(0..3, 0..1);
// }
