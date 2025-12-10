pub mod buffers;
pub mod components;
pub mod enums;
pub mod material_types;
pub mod resources;
mod state;

use crate::core::state::EngineState;

pub use self::state::RenderState;

/// Flush dirty components to GPU buffers
/// Updates uniform buffers for all components marked as dirty
pub fn flush_components(engine_state: &mut EngineState) {
    // Get queue
    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Flush each window's render state
    for (_window_id, window_state) in engine_state.windows.iter_mut() {
        let render_state = match window_state.render_state.as_mut() {
            Some(rs) => rs,
            None => continue,
        };

        // Flush dirty cameras
        for (component_id, camera) in render_state.components.cameras.iter_mut() {
            if camera.is_dirty {
                // Pack camera uniform data (view + proj matrices)
                let mut camera_data = Vec::with_capacity(128);
                camera_data.extend_from_slice(bytemuck::bytes_of(&camera.view_mat));
                camera_data.extend_from_slice(bytemuck::bytes_of(&camera.proj_mat));

                // Write to GPU
                let _ = render_state.uniform_buffer_manager.write_data(
                    queue,
                    *component_id,
                    camera.camera_uniform_offset,
                    &camera_data,
                );

                // Clear dirty flag
                camera.is_dirty = false;
            }
        }

        // Flush dirty models
        for (component_id, model) in render_state.components.models.iter_mut() {
            if model.is_dirty {
                // Pack model uniform data (model matrix)
                let model_data = bytemuck::bytes_of(&model.model_mat);

                // Write to GPU
                let _ = render_state.uniform_buffer_manager.write_data(
                    queue,
                    *component_id,
                    model.model_uniform_offset,
                    model_data,
                );

                // Clear dirty flag
                model.is_dirty = false;
            }
        }
    }
}

pub fn render_frames(engine_state: &mut EngineState) {
    // Get device and queue
    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Render all windows
    for (_window_id, window_state) in engine_state.windows.iter_mut() {
        // Ensure render state exists
        if window_state.render_state.is_none() {
            window_state.render_state = Some(RenderState::new());
        }

        let render_state = window_state.render_state.as_ref().unwrap();

        // Get the surface texture
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        // Create a texture view
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Create a render pass with cached clear color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

        // Submit the commands
        queue.submit(std::iter::once(encoder.finish()));

        // Present the frame
        surface_texture.present();
    }
}
