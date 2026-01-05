mod branches;

use crate::core::render::RenderState;
use crate::core::resources::MATERIAL_FALLBACK_ID;

pub fn pass_forward(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let scene = &render_state.scene;

    let vertex_sys = match render_state.vertex.as_mut() {
        Some(v) => v,
        None => return,
    };

    let bindings = match render_state.bindings.as_ref() {
        Some(b) => b,
        None => return,
    };

    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let cache = &mut render_state.cache;
    let depth_target = render_state.forward_depth_target.as_ref();
    let light_system = match render_state.light_system.as_mut() {
        Some(sys) => sys,
        None => return,
    };
    let materials_lambert = &render_state.scene.materials_lambert;
    let materials_unlit = &render_state.scene.materials_unlit;

    // 1. Sort cameras by order

    let mut sorted_cameras: Vec<_> = scene.cameras.iter().collect();

    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (camera_index, (camera_id, camera_record)) in sorted_cameras.into_iter().enumerate() {
        light_system.write_draw_params(camera_index as u32, light_system.max_lights_per_camera);

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

                depth_stencil_attachment: depth_target.map(|target| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &target.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 4. Bind Shared (Group 0: Frame + Camera)
            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(*camera_id) as u32;
                let light_offset = light_system.draw_params_offset(camera_index as u32) as u32;
                render_pass.set_bind_group(0, shared_group, &[camera_offset, light_offset]);
            }

            // 5. Filter and draw models (Unlit/Lambert Opaque branches)
            for (model_id, model_record) in &scene.models {
                // Check layer mask
                if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                    continue;
                }

                let unlit_material_id = model_record
                    .material_id
                    .filter(|id| materials_unlit.contains_key(id));

                if let Some(material_id) = unlit_material_id {
                    if let Some(unlit_group) = bindings.unlit_group.as_ref() {
                        let model_offset = bindings.model_pool.get_offset(*model_id) as u32;
                        let material_offset =
                            bindings.material_unlit_pool.get_offset(material_id) as u32;
                        render_pass.set_bind_group(1, unlit_group, &[model_offset, material_offset]);
                    }
                    let pipeline =
                        branches::unlit::get_pipeline(cache, frame_index, device, library);
                    render_pass.set_pipeline(pipeline);
                } else {
                    let material_id = model_record
                        .material_id
                        .filter(|id| materials_lambert.contains_key(id))
                        .unwrap_or(MATERIAL_FALLBACK_ID);

                    if let Some(lambert_group) = bindings.lambert_group.as_ref() {
                        let model_offset = bindings.model_pool.get_offset(*model_id) as u32;
                        let material_offset =
                            bindings.material_lambert_pool.get_offset(material_id) as u32;
                        render_pass.set_bind_group(1, lambert_group, &[model_offset, material_offset]);
                    }
                    let pipeline =
                        branches::lambert::get_pipeline(cache, frame_index, device, library);
                    render_pass.set_pipeline(pipeline);
                }

                // Bind Geometry
                if let Ok(Some(index_info)) = vertex_sys.index_info(model_record.geometry_id) {
                    let _ = vertex_sys.bind(&mut render_pass, model_record.geometry_id);

                    render_pass.draw_indexed(0..index_info.count, 0, 0..1);
                }
            }
        }
    }
}
