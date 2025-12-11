pub mod allocator;
pub mod binding;
pub mod buffers;
pub mod components;
pub mod enums;
pub mod material_types;
pub mod passes;
pub mod pipeline;
pub mod resources;
mod state;

use crate::core::state::EngineState;

use self::passes::{ForwardRenderItem, compose_pass, forward_pass};
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

    // Convert time values from ms to seconds
    let time = engine_state.time as f32 / 1000.0;
    let delta_time = engine_state.delta_time as f32 / 1000.0;

    // Render all windows
    for (_window_id, window_state) in engine_state.windows.iter_mut() {
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
        let render_state = &mut window_state.render_state;

        // Phase 1: Collect render items and update bindings (lazy)
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

                render_items.push(ForwardRenderItem {
                    camera_id: *camera_id,
                    model_id: *model_id,
                    shader_id,
                    material_id: model.material,
                    geometry_id: model.geometry,
                });
            }
        }

        // Update bindings for all render items
        for item in &render_items {
            // Create binding key
            let binding_key = binding::BindingKey {
                component_id: item.model_id,
                shader_id: item.shader_id,
                resource_ids: vec![item.material_id, item.geometry_id],
            };

            // Get camera and model
            let camera = render_state
                .components
                .cameras
                .get(&item.camera_id)
                .unwrap();
            let model = render_state.components.models.get(&item.model_id).unwrap();

            // Get material uniforms
            let material_uniforms = render_state
                .resources
                .materials
                .get(&item.material_id)
                .map(|m| &m.uniform_values);

            // Get shader (mutable)
            let shader = match render_state.resources.shaders.get_mut(&item.shader_id) {
                Some(s) => s,
                None => continue,
            };

            // Get or update bindings (lazy)
            if let Err(e) = render_state.binding_manager.get_or_update(
                &binding_key,
                device,
                queue,
                shader,
                time,
                delta_time,
                Some(camera),
                Some(model),
                material_uniforms,
            ) {
                log::error!("Failed to get/update bindings: {}", e);
            }
        }

        // Phase 2: Render passes
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Collect camera IDs first to avoid borrow checker issues
        let camera_ids: Vec<_> = render_state.components.cameras.keys().copied().collect();

        // Forward pass: Render models to camera render targets
        for camera_id in camera_ids {
            forward_pass(&mut encoder, device, render_state, camera_id, &render_items);
        }

        // Compose pass: Blit camera render targets to surface
        compose_pass(&mut encoder, device, &view, render_state);

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
