use wgpu;

use crate::core::render::RenderState;

/// Compose pass - blits camera render targets to the surface
///
/// This pass composites all camera render targets onto the final surface:
/// 1. Clears surface with clear_color
/// 2. For each camera (sorted by component_id), blits render target to surface using fullscreen quad
///
/// # Arguments
/// * `encoder` - Command encoder for recording commands
/// * `device` - WGPU device for creating bind groups
/// * `surface_view` - Surface texture view to render to
/// * `render_state` - Render state with cameras and blit resources
pub fn compose_pass(
    encoder: &mut wgpu::CommandEncoder,
    device: &wgpu::Device,
    surface_view: &wgpu::TextureView,
    render_state: &RenderState,
) {
    // Get blit resources
    let blit_pipeline = match &render_state.blit_pipeline {
        Some(p) => p,
        None => {
            log::warn!("Blit pipeline not initialized, skipping compose pass");
            return;
        }
    };

    let blit_sampler = match &render_state.blit_sampler {
        Some(s) => s,
        None => {
            log::warn!("Blit sampler not initialized, skipping compose pass");
            return;
        }
    };

    let bind_group_layout = match &render_state.blit_bind_group_layout {
        Some(l) => l,
        None => {
            log::warn!("Blit bind group layout not initialized, skipping compose pass");
            return;
        }
    };

    // Sort camera IDs for deterministic rendering order
    let mut camera_ids: Vec<_> = render_state.components.cameras.keys().copied().collect();
    camera_ids.sort();

    // Begin render pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(render_state.clear_color),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    // Set pipeline once
    render_pass.set_pipeline(blit_pipeline);

    // Blit each camera's render target
    for camera_id in camera_ids {
        let camera = match render_state.components.cameras.get(&camera_id) {
            Some(c) => c,
            None => continue,
        };

        // Create bind group for this camera's render target
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Blit Bind Group - Camera {}", camera_id)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&camera.render_target_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(blit_sampler),
                },
            ],
        });

        // Set bind group and draw fullscreen triangle
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
