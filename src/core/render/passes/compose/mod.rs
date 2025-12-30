use crate::core::render::RenderState;

pub fn pass_compose(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface_texture: &wgpu::SurfaceTexture,
    config: &wgpu::SurfaceConfiguration,
) {
    let view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // 1. Sort cameras by order
    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    // 2. Get or Create Compose Pipeline
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    if render_state.passes.compose.pipeline.is_none() {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compose Pipeline Layout"),
            bind_group_layouts: &[&library.layout_target],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Compose Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &library.compose_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &library.compose_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        render_state.passes.compose.pipeline = Some(pipeline);
    }

    // 3. Begin compose pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    if let Some(pipeline) = &render_state.passes.compose.pipeline {
        render_pass.set_pipeline(pipeline);

        for (_id, record) in sorted_cameras {
            let target = match &record.render_target {
                Some(t) => t,
                None => continue,
            };

            // 4. Resolve viewport
            let (x, y) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_position(config.width, config.height))
                .unwrap_or((0, 0));

            let (width, height) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(config.width, config.height))
                .unwrap_or((config.width, config.height));

            render_pass.set_viewport(x as f32, y as f32, width as f32, height as f32, 0.0, 1.0);

            // 5. Create Bind Group for this camera's target
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Compose Bind Group"),
                layout: &library.layout_target,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                ],
            });

            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}
