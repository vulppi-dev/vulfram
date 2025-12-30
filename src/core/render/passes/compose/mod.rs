use crate::core::render::RenderState;

pub fn pass_compose(
    render_state: &mut RenderState,
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

    // 2. Begin compose pass
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

    for (_id, record) in sorted_cameras {
        let _target = match &record.render_target {
            Some(t) => t,
            None => continue,
        };

        // 3. Resolve viewport
        let (x, y) = record
            .view_position
            .as_ref()
            .map(|vp: &crate::core::resources::ViewPosition| {
                vp.resolve_position(config.width, config.height)
            })
            .unwrap_or((0, 0));

        let (width, height) = record
            .view_position
            .as_ref()
            .map(|vp: &crate::core::resources::ViewPosition| {
                vp.resolve_size(config.width, config.height)
            })
            .unwrap_or((config.width, config.height));

        render_pass.set_viewport(x as f32, y as f32, width as f32, height as f32, 0.0, 1.0);

        // TODO: Bind camera target texture and draw fullscreen triangle/quad
        // render_pass.set_pipeline(...);
        // render_pass.set_bind_group(0, ...);
        // render_pass.draw(0..3, 0..1);
    }
}
