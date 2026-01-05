mod branches;

use crate::core::render::RenderState;
use crate::core::resources::{MATERIAL_FALLBACK_ID, SurfaceType};

pub fn pass_forward(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    struct DrawItem {
        model_id: u32,
        geometry_id: u32,
        material_id: u32,
    }

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
    let materials_standard = &render_state.scene.materials_standard;

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

            // 5. Filter and group models by surface type
            let mut opaque = Vec::new();
            let mut masked = Vec::new();
            let mut transparent = Vec::new();

            for (model_id, model_record) in &scene.models {
                if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                    continue;
                }

                let material_id = model_record
                    .material_id
                    .filter(|id| materials_standard.contains_key(id))
                    .unwrap_or(MATERIAL_FALLBACK_ID);

                let surface_type = materials_standard
                    .get(&material_id)
                    .map(|record| record.surface_type)
                    .unwrap_or(SurfaceType::Opaque);

                let item = DrawItem {
                    model_id: *model_id,
                    geometry_id: model_record.geometry_id,
                    material_id,
                };

                match surface_type {
                    SurfaceType::Opaque => opaque.push(item),
                    SurfaceType::Masked => masked.push(item),
                    SurfaceType::Transparent => transparent.push(item),
                }
            }

            let pipeline =
                branches::standard::get_pipeline(cache, frame_index, device, library);
            render_pass.set_pipeline(pipeline);

            let mut draw_items = |items: Vec<DrawItem>| {
                for item in items {
                    if let Some(standard_group) = bindings.standard_group.as_ref() {
                        let model_offset = bindings.model_pool.get_offset(item.model_id) as u32;
                        let material_offset =
                            bindings.material_standard_pool.get_offset(item.material_id) as u32;
                        render_pass
                            .set_bind_group(1, standard_group, &[model_offset, material_offset]);
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(item.geometry_id) {
                        let _ = vertex_sys.bind(&mut render_pass, item.geometry_id);
                        render_pass.draw_indexed(0..index_info.count, 0, 0..1);
                    }
                }
            };

            draw_items(opaque);
            draw_items(masked);
            draw_items(transparent);
        }
    }
}
