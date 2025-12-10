pub mod allocator;
pub mod binding;
pub mod buffers;
pub mod components;
pub mod enums;
pub mod material_types;
pub mod pipeline;
pub mod resources;
mod state;

use crate::core::state::EngineState;

pub use self::state::RenderState;

/// Flush dirty components to GPU buffers (DEPRECATED)
/// This function is now a no-op. Updates happen lazily during rendering.
pub fn flush_components(_engine_state: &mut EngineState) {
    // Updates now happen lazily during rendering via BindingManager::get_or_update()
    // Dirty flags are cleared after render_frame completes in Phase 3
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

        // Split render state to avoid borrow checker issues
        let render_state = window_state.render_state.as_mut().unwrap();

        // Phase 1: Update all bindings (lazy)
        // Collect (camera_id, model_id, shader_id, material_id, geometry_id) tuples
        let mut render_items = Vec::new();

        for (camera_id, camera) in render_state.components.cameras.iter() {
            for (model_id, model) in render_state.components.models.iter() {
                // Layer mask filtering
                if (camera.layer_mask & model.layer_mask) == 0 {
                    continue;
                }

                // Validate resources exist
                let material = match render_state.resources.materials.get(&model.material) {
                    Some(m) => m,
                    None => continue,
                };

                if !render_state
                    .resources
                    .geometries
                    .contains_key(&model.geometry)
                {
                    continue;
                }

                let shader_id = material.pipeline_spec.shader_id;

                render_items.push((
                    *camera_id,
                    *model_id,
                    shader_id,
                    model.material,
                    model.geometry,
                ));
            }
        }

        // Update bindings for all render items
        for (camera_id, model_id, shader_id, material_id, geometry_id) in &render_items {
            // Create binding key
            let binding_key = binding::BindingKey {
                component_id: *model_id,
                shader_id: *shader_id,
                resource_ids: vec![*material_id, *geometry_id],
            };

            // Get camera and model
            let camera = render_state.components.cameras.get(camera_id).unwrap();
            let model = render_state.components.models.get(model_id).unwrap();

            // Get shader (mutable)
            let shader = match render_state.resources.shaders.get_mut(shader_id) {
                Some(s) => s,
                None => continue,
            };

            // Get or update bindings (lazy)
            if let Err(e) = render_state.binding_manager.get_or_update(
                &binding_key,
                device,
                queue,
                shader,
                Some(camera),
                Some(model),
            ) {
                log::error!("Failed to get/update bindings: {}", e);
            }
        }

        // Phase 2: Create command encoder and render
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Iterate through cameras for rendering
        for (camera_id, camera) in render_state.components.cameras.iter() {
            // Create render pass for this camera
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Camera {} Render Pass", camera_id)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &camera.render_target_view,
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

            // TODO: Actual draw calls
            // Filter render_items for this camera
            // For each item:
            //   - Get binding from binding_manager
            //   - Get/create pipeline from pipeline_cache
            //   - Set pipeline, bind groups, vertex/index buffers
            //   - Draw indexed
        }

        // Copy camera render targets to surface (blit)
        // TODO: Implement camera render target to surface copy
        // For now, just clear the surface
        {
            let _final_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surface Clear Pass"),
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

        // Phase 3: Clear dirty flags after render
        for camera in render_state.components.cameras.values_mut() {
            camera.is_dirty = false;
        }
        for model in render_state.components.models.values_mut() {
            model.is_dirty = false;
        }
    }
}
