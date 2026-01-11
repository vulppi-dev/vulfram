mod branches;

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::geometry::Frustum;
use crate::core::resources::{MATERIAL_FALLBACK_ID, SurfaceType};

pub fn pass_forward(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let mut instance_cursor: u32;

    let scene = &render_state.scene;

    let vertex_sys = match render_state.vertex.as_mut() {
        Some(v) => v,
        None => return,
    };

    let bindings = match render_state.bindings.as_mut() {
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
    render_state.gizmos.prepare(device, queue);
    let materials_standard = &render_state.scene.materials_standard;
    let materials_pbr = &render_state.scene.materials_pbr;

    // Pre-cache Gizmo Pipeline once per pass if needed
    let gizmo_pipeline_key = if !render_state.gizmos.is_empty() {
        Some(PipelineKey {
            shader_id: ShaderId::Gizmo as u64,
            color_format: wgpu::TextureFormat::Rgba16Float,
            depth_format: depth_target.map(|t| t.format),
            sample_count: 1,
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Greater,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        })
    } else {
        None
    };

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

        // Reset collector for this camera
        render_state.collector.clear();
        instance_cursor = 0;

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
                            load: wgpu::LoadOp::Clear(0.0), // Reverse Z
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            vertex_sys.begin_pass();

            // 4. Bind Shared (Group 0: Frame + Camera + ModelPool)
            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(*camera_id) as u32;
                let light_offset = light_system.draw_params_offset(camera_index as u32) as u32;
                render_pass.set_bind_group(0, shared_group, &[camera_offset, light_offset]);
            }

            // 5. Filter and group models by surface type
            let frustum = Frustum::from_view_projection(camera_record.data.view_projection);
            let collector = &mut render_state.collector;

            for (model_id, model_record) in &scene.models {
                if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                    continue;
                }

                if let Some(aabb) = vertex_sys.aabb(model_record.geometry_id) {
                    let world_aabb = aabb.transform(&model_record.data.transform);
                    if !frustum.intersects_aabb(world_aabb.min, world_aabb.max) {
                        continue;
                    }
                }

                let material_id = model_record.material_id.unwrap_or(MATERIAL_FALLBACK_ID);

                let model_depth = {
                    let clip = camera_record.data.view_projection * model_record.data.translation;
                    if clip.w.abs() > 1e-5 {
                        clip.z / clip.w
                    } else {
                        0.0
                    }
                };

                use crate::core::render::state::DrawItem;

                if let Some(record) = materials_pbr.get(&material_id) {
                    let item = DrawItem {
                        model_id: *model_id,
                        geometry_id: model_record.geometry_id,
                        material_id,
                        depth: model_depth,
                        instance_idx: 0,
                    };
                    match record.surface_type {
                        SurfaceType::Opaque => collector.pbr_opaque.push(item),
                        SurfaceType::Masked => collector.pbr_masked.push(item),
                        SurfaceType::Transparent => collector.pbr_transparent.push(item),
                    }
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
                    depth: model_depth,
                    instance_idx: 0,
                };

                match surface_type {
                    SurfaceType::Opaque => collector.standard_opaque.push(item),
                    SurfaceType::Masked => collector.standard_masked.push(item),
                    SurfaceType::Transparent => collector.standard_transparent.push(item),
                }
            }

            // 6. Sort and prepare instance data per branch
            collector
                .pbr_opaque
                .sort_by_key(|a| (a.material_id, a.geometry_id));
            collector
                .standard_opaque
                .sort_by_key(|a| (a.material_id, a.geometry_id));
            collector
                .pbr_masked
                .sort_by_key(|a| (a.material_id, a.geometry_id));
            collector
                .standard_masked
                .sort_by_key(|a| (a.material_id, a.geometry_id));

            // Sort Far-to-Near (Painter's Algorithm)
            // With Reverse Z: Far is 0.0, Near is 1.0. So we sort Ascending.
            collector.standard_transparent.sort_by(|a, b| {
                a.depth
                    .partial_cmp(&b.depth)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            collector.pbr_transparent.sort_by(|a, b| {
                a.depth
                    .partial_cmp(&b.depth)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let groups = [
                &mut collector.pbr_opaque,
                &mut collector.standard_opaque,
                &mut collector.pbr_masked,
                &mut collector.standard_masked,
                &mut collector.pbr_transparent,
                &mut collector.standard_transparent,
            ];

            for group in groups {
                for item in group.iter_mut() {
                    item.instance_idx = instance_cursor;
                    if let Some(record) = scene.models.get(&item.model_id) {
                        collector.instance_data.push(record.data);
                        instance_cursor += 1;
                    }
                }
            }

            if !collector.instance_data.is_empty() {
                bindings
                    .instance_pool
                    .write_slice(0, &collector.instance_data);
            }

            let pbr_pipeline = branches::pbr::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Opaque,
            );
            render_pass.set_pipeline(pbr_pipeline);
            {
                let mut i = 0;
                while i < collector.pbr_opaque.len() {
                    let batch_start = i;
                    let item = &collector.pbr_opaque[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.pbr_opaque.len()
                        && collector.pbr_opaque[i].material_id == mat_id
                        && collector.pbr_opaque[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_pbr.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_pbr_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance = collector.pbr_opaque[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            let pbr_pipeline = branches::pbr::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Masked,
            );
            render_pass.set_pipeline(pbr_pipeline);
            {
                let mut i = 0;
                while i < collector.pbr_masked.len() {
                    let batch_start = i;
                    let item = &collector.pbr_masked[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.pbr_masked.len()
                        && collector.pbr_masked[i].material_id == mat_id
                        && collector.pbr_masked[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_pbr.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_pbr_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance = collector.pbr_masked[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            let pipeline = branches::standard::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Opaque,
            );
            render_pass.set_pipeline(pipeline);
            {
                let mut i = 0;
                while i < collector.standard_opaque.len() {
                    let batch_start = i;
                    let item = &collector.standard_opaque[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.standard_opaque.len()
                        && collector.standard_opaque[i].material_id == mat_id
                        && collector.standard_opaque[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_standard.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_standard_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance =
                                collector.standard_opaque[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            let pipeline = branches::standard::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Masked,
            );
            render_pass.set_pipeline(pipeline);
            {
                let mut i = 0;
                while i < collector.standard_masked.len() {
                    let batch_start = i;
                    let item = &collector.standard_masked[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.standard_masked.len()
                        && collector.standard_masked[i].material_id == mat_id
                        && collector.standard_masked[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_standard.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_standard_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance =
                                collector.standard_masked[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            let pbr_pipeline = branches::pbr::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Transparent,
            );
            render_pass.set_pipeline(pbr_pipeline);
            {
                let mut i = 0;
                while i < collector.pbr_transparent.len() {
                    let batch_start = i;
                    let item = &collector.pbr_transparent[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.pbr_transparent.len()
                        && collector.pbr_transparent[i].material_id == mat_id
                        && collector.pbr_transparent[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_pbr.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_pbr_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance =
                                collector.pbr_transparent[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            let pipeline = branches::standard::get_pipeline(
                cache,
                frame_index,
                device,
                library,
                SurfaceType::Transparent,
            );
            render_pass.set_pipeline(pipeline);
            {
                let mut i = 0;
                while i < collector.standard_transparent.len() {
                    let batch_start = i;
                    let item = &collector.standard_transparent[i];
                    let mat_id = item.material_id;
                    let geom_id = item.geometry_id;

                    while i < collector.standard_transparent.len()
                        && collector.standard_transparent[i].material_id == mat_id
                        && collector.standard_transparent[i].geometry_id == geom_id
                    {
                        i += 1;
                    }
                    let batch_count = (i - batch_start) as u32;

                    if let Some(material) = scene.materials_standard.get(&mat_id) {
                        if let Some(group) = material.bind_group.as_ref() {
                            let material_offset =
                                bindings.material_standard_pool.get_offset(mat_id) as u32;
                            render_pass.set_bind_group(1, group, &[material_offset]);
                        }
                    }

                    if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                        if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                            let first_instance =
                                collector.standard_transparent[batch_start].instance_idx;
                            render_pass.draw_indexed(
                                0..index_info.count,
                                0,
                                first_instance..(first_instance + batch_count),
                            );
                        }
                    }
                }
            }

            // Draw Gizmos
            if let Some(key) = gizmo_pipeline_key {
                let pipeline = cache.get_or_create(key, frame_index, || {
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Gizmo Pipeline"),
                        layout: Some(&library.gizmo_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &library.gizmo_shader,
                            entry_point: Some("vs_main"),
                            buffers: &[wgpu::VertexBufferLayout {
                                array_stride: std::mem::size_of::<
                                    crate::core::render::gizmos::GizmoVertex,
                                >() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    },
                                    wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x4,
                                        offset: 16, // Changed from 12 to 16 due to padding
                                        shader_location: 1,
                                    },
                                ],
                            }],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &library.gizmo_shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Rgba16Float,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::LineList,
                            ..Default::default()
                        },
                        depth_stencil: depth_target.map(|target| wgpu::DepthStencilState {
                            format: target.format,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Greater,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview_mask: None,
                        cache: None,
                    })
                });
                render_pass.set_pipeline(pipeline);
                render_state.gizmos.draw(&mut render_pass);
            }
        }
    }
}
