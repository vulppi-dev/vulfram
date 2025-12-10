use wgpu;

use crate::core::render::RenderState;

/// Compose pass - blits camera render targets to the surface
///
/// This pass composites all camera render targets onto the final surface:
/// 1. Clears surface with clear_color
/// 2. For each camera (sorted by component_id for deterministic order):
///    - Copies camera render target texture to surface texture
///
/// **Current limitation**: Uses `as_image_copy()` which always copies to origin (0,0).
/// Viewport positioning is calculated but not applied. All cameras render to full surface.
/// For proper viewport positioning and blending, would need fullscreen quad shader approach.
///
/// # Arguments
/// * `encoder` - Command encoder for recording commands
/// * `surface_view` - Surface texture view to render to
/// * `surface_texture` - Surface texture for copy operations  
/// * `surface_size` - Surface dimensions (width, height)
/// * `render_state` - Render state with cameras and clear color
pub fn compose_pass(
    encoder: &mut wgpu::CommandEncoder,
    surface_view: &wgpu::TextureView,
    surface_texture: &wgpu::Texture,
    surface_size: (u32, u32),
    render_state: &RenderState,
) {
    // Step 1: Clear the surface with clear_color
    {
        let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Compose Pass - Clear"),
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
    }

    // Step 2: Copy camera render targets to surface
    // Sort cameras by component_id for deterministic rendering order
    let mut camera_ids: Vec<_> = render_state.components.cameras.keys().copied().collect();
    camera_ids.sort();

    for camera_id in camera_ids {
        let camera = match render_state.components.cameras.get(&camera_id) {
            Some(c) => c,
            None => continue,
        };

        // Calculate viewport rectangle in pixels
        let (position, size) = camera
            .viewport
            .calculate_rect(surface_size.0, surface_size.1);

        // Get render target size
        let rt_size = camera.render_target.size();

        // Copy camera render target to surface
        // Uses the smaller of (viewport size, render target size, surface size)
        let copy_width = rt_size.width.min(size.x).min(surface_size.0);
        let copy_height = rt_size.height.min(size.y).min(surface_size.1);

        if copy_width > 0 && copy_height > 0 {
            encoder.copy_texture_to_texture(
                camera.render_target.as_image_copy(),
                surface_texture.as_image_copy(),
                wgpu::Extent3d {
                    width: copy_width,
                    height: copy_height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}
