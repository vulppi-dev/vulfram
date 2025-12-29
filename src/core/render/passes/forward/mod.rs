use crate::core::render::RenderState;

pub fn pass_forward(
    render_state: &mut RenderState,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
) {
    // 1. Sort cameras by order
    let mut sorted_cameras: Vec<_> = render_state.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (camera_id, camera_record) in sorted_cameras {
        // 2. Get render target view
        let target_view = match &camera_record.render_target {
            Some(target) => &target.view,
            None => continue,
        };

        // 3. Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Forward Pass - Camera {}", camera_id)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None, // TODO: Add depth buffer
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 4. Bind Shared (Group 0: Frame + Camera)
            if let (Some(bg_shared), Some(pool)) =
                (&render_state.bind_group_shared, &render_state.camera_buffer)
            {
                let offset = pool.get_offset(*camera_id) as u32;
                render_pass.set_bind_group(0, bg_shared, &[offset]);
            }

            // 5. Filter and draw models
            for (model_id, model_record) in &render_state.models {
                // Check layer mask
                if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                    continue;
                }

                // Bind Object (Group 1: Model)
                if let (Some(bg_object), Some(pool)) =
                    (&render_state.bind_group_object, &render_state.model_buffer)
                {
                    let offset = pool.get_offset(*model_id) as u32;
                    render_pass.set_bind_group(1, bg_object, &[offset]);
                }

                // TODO: Set vertex buffers from VertexAllocatorSystem
                // TODO: Get pipeline from RenderCache
                // TODO: Draw
            }
        }
    }
}
