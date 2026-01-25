use crate::core::render::RenderState;
use crate::core::resources::SkyboxMode;

pub fn pass_skybox(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let skybox = &render_state.environment.skybox;
    if matches!(skybox.mode, SkyboxMode::None) {
        return;
    }

    let sample_count = render_state.msaa_sample_count();

    if let Some((_, camera)) = render_state.scene.cameras.iter().next() {
        if let Some(target) = &camera.render_target {
            let size = target._texture.size();
            let needs_msaa = sample_count > 1
                && match render_state.forward_msaa_target.as_ref() {
                    Some(existing) => {
                        let existing_size = existing._texture.size();
                        existing_size.width != size.width
                            || existing_size.height != size.height
                            || existing.sample_count != sample_count
                    }
                    None => true,
                };

            if needs_msaa {
                render_state.forward_msaa_target =
                    Some(crate::core::resources::RenderTarget::new_with_samples(
                        device,
                        size,
                        wgpu::TextureFormat::Rgba16Float,
                        sample_count,
                    ));
            }
        }
    }

    let tint = skybox.tint * skybox.intensity;
    let clear_color = wgpu::Color {
        r: tint.x as f64,
        g: tint.y as f64,
        b: tint.z as f64,
        a: 1.0,
    };

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (_camera_id, camera_record) in sorted_cameras {
        let target_view = match &camera_record.render_target {
            Some(target) => &target.view,
            None => continue,
        };

        let (color_view, resolve_target) = if sample_count > 1 {
            match render_state.forward_msaa_target.as_ref() {
                Some(msaa) => (&msaa.view, Some(target_view)),
                None => (target_view, None),
            }
        } else {
            (target_view, None)
        };

        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Skybox Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let _ = frame_index;
    }
}
