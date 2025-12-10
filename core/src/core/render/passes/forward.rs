use wgpu;

use crate::core::render::{
    RenderState,
    binding::BindingKey,
    components::ComponentId,
    resources::{GeometryId, MaterialId, ShaderId},
};

/// Render item for forward pass
#[derive(Debug, Clone, Copy)]
pub struct ForwardRenderItem {
    pub camera_id: ComponentId,
    pub model_id: ComponentId,
    pub shader_id: ShaderId,
    pub material_id: MaterialId,
    pub geometry_id: GeometryId,
}

/// Forward rendering pass - draws models to camera render targets
///
/// This pass renders all visible models for a specific camera using forward rendering:
/// - Iterates through render items filtered by camera
/// - Gets/creates pipelines lazily
/// - Sets up bind groups and buffers
/// - Issues draw indexed calls
///
/// # Arguments
/// * `encoder` - Command encoder for recording commands
/// * `device` - WGPU device for pipeline creation
/// * `render_state` - Mutable render state with all resources
/// * `camera_id` - Camera to render for
/// * `render_items` - All render items (will be filtered by camera_id)
pub fn forward_pass(
    encoder: &mut wgpu::CommandEncoder,
    device: &wgpu::Device,
    render_state: &mut RenderState,
    camera_id: ComponentId,
    render_items: &[ForwardRenderItem],
) {
    // Get camera from render state
    let camera = match render_state.components.cameras.get(&camera_id) {
        Some(c) => c,
        None => {
            log::warn!("Camera {} not found, skipping forward pass", camera_id);
            return;
        }
    };

    // Extract what we need from camera before mutable borrows
    let render_target_view = camera.render_target_view.clone();
    let clear_color = render_state.clear_color;

    // Begin render pass for this camera
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(&format!("Forward Pass - Camera {}", camera_id)),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &render_target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    // Filter render items for this camera
    let camera_items: Vec<_> = render_items
        .iter()
        .filter(|item| item.camera_id == camera_id)
        .collect();

    // Draw each model
    for item in camera_items {
        // Get resources
        let shader = match render_state.resources.shaders.get(&item.shader_id) {
            Some(s) => s,
            None => {
                log::warn!("Shader {} not found, skipping draw", item.shader_id);
                continue;
            }
        };

        let material = match render_state.resources.materials.get(&item.material_id) {
            Some(m) => m,
            None => {
                log::warn!("Material {} not found, skipping draw", item.material_id);
                continue;
            }
        };

        let geometry = match render_state.resources.geometries.get(&item.geometry_id) {
            Some(g) => g,
            None => {
                log::warn!("Geometry {} not found, skipping draw", item.geometry_id);
                continue;
            }
        };

        // Get or create pipeline (lazy)
        let pipeline = match render_state.pipeline_cache.get_or_create(
            item.shader_id,
            item.material_id,
            device,
            shader,
            material,
            render_state.surface_format,
        ) {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "Failed to create pipeline for shader {} material {}: {}",
                    item.shader_id,
                    item.material_id,
                    e
                );
                continue;
            }
        };

        // Get binding
        let binding_key = BindingKey {
            component_id: item.model_id,
            shader_id: item.shader_id,
            resource_ids: vec![item.material_id, item.geometry_id],
        };

        let binding = match render_state.binding_manager.get(&binding_key) {
            Some(b) => b,
            None => {
                log::warn!(
                    "Binding not found for model {} shader {} (should have been created in Phase 1)",
                    item.model_id,
                    item.shader_id
                );
                continue;
            }
        };

        // Set pipeline
        render_pass.set_pipeline(pipeline);

        // Set bind groups
        // Group 0: Camera/Global uniforms
        if let Some(bind_group) = &binding.bind_group_0 {
            let offsets = if let Some(offset) = binding.group_0_offset {
                vec![offset as u32]
            } else {
                vec![]
            };
            render_pass.set_bind_group(0, bind_group, &offsets);
        }

        // Group 1: Material uniforms (if exists)
        if let Some(bind_group) = &binding.bind_group_1 {
            let offsets = if let Some(offset) = binding.group_1_offset {
                vec![offset as u32]
            } else {
                vec![]
            };
            render_pass.set_bind_group(1, bind_group, &offsets);
        }

        // Group 2: Model/Instance uniforms
        if let Some(bind_group) = &binding.bind_group_2 {
            let offsets = if let Some(offset) = binding.group_2_offset {
                vec![offset as u32]
            } else {
                vec![]
            };
            render_pass.set_bind_group(2, bind_group, &offsets);
        }

        // Set vertex buffer
        render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));

        // Set index buffer and draw
        render_pass.set_index_buffer(
            geometry.index_buffer.slice(..),
            match geometry.index_format {
                crate::core::render::enums::IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
                crate::core::render::enums::IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
            },
        );

        // Draw indexed
        render_pass.draw_indexed(0..geometry.index_count, 0, 0..1);
    }
}
