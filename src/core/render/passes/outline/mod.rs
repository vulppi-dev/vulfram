use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::geometry::Frustum;
use crate::core::resources::VertexStream;

pub fn pass_outline(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let scene = &render_state.scene;
    if scene.cameras.is_empty() {
        return;
    }

    let library = match render_state.library.as_ref() {
        Some(lib) => lib,
        None => return,
    };

    let (bindings, vertex_sys, light_system, collector) = match (
        render_state.bindings.as_mut(),
        render_state.vertex.as_mut(),
        render_state.light_system.as_mut(),
    ) {
        (Some(bindings), Some(vertex_sys), Some(light_system)) => {
            (bindings, vertex_sys, light_system, &mut render_state.collector)
        }
        _ => return,
    };


    let mut sorted_cameras: Vec<_> = scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (camera_index, (camera_id, camera_record)) in sorted_cameras.into_iter().enumerate() {
        let outline_target = match &camera_record.outline_target {
            Some(target) => target,
            None => continue,
        };

        light_system.write_draw_params(camera_index as u32, light_system.max_lights_per_camera);

        let frustum = Frustum::from_view_projection(camera_record.data.view_projection);
        collector.outline_items.clear();
        collector.outline_instance_data.clear();
        let mut instance_cursor = 0u32;

        for (_model_id, model_record) in &scene.models {
            if !model_record.cast_outline {
                continue;
            }
            if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                continue;
            }
            if vertex_sys
                .index_info(model_record.geometry_id)
                .ok()
                .flatten()
                .is_none()
            {
                continue;
            }
            if let Some(aabb) = vertex_sys.aabb(model_record.geometry_id) {
                let world_aabb = aabb.transform(&model_record.data.transform);
                if !frustum.intersects_aabb(world_aabb.min, world_aabb.max) {
                    continue;
                }
            }

            collector
                .outline_items
                .push((model_record.geometry_id, instance_cursor));
            collector.outline_instance_data.push(model_record.data);
            instance_cursor += 1;
        }

        if collector.outline_items.is_empty() {
            continue;
        }

        collector.outline_items.sort_by_key(|item| item.0);

        bindings
            .outline_instance_pool
            .write_slice(0, &collector.outline_instance_data);

        let key = PipelineKey {
            shader_id: ShaderId::Outline as u64,
            color_format: outline_target.format,
            depth_format: None,
            sample_count: 1,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        };

        let pipeline = render_state.cache.get_or_create(key, frame_index, || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Outline Pipeline"),
                layout: Some(&library.outline_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &library.outline_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Position.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Normal.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 1,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Tangent.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 2,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Color0.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 3,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::UV0.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 4,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::UV1.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 5,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Joints.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Uint16x4,
                                offset: 0,
                                shader_location: 6,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Weights.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 7,
                            }],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &library.outline_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: outline_target.format,
                        blend: key.blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Outline Pass - Camera {}", camera_id)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &outline_target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some(shared_group) = bindings.shared_group.as_ref() {
            let camera_offset = bindings.camera_pool.get_offset(*camera_id) as u32;
            let light_offset = light_system.draw_params_offset(camera_index as u32) as u32;
            render_pass.set_bind_group(0, shared_group, &[camera_offset, light_offset]);
        }

        if let Some(model_group) = bindings.outline_model_bind_group.as_ref() {
            render_pass.set_bind_group(1, model_group, &[]);
        }

        render_pass.set_pipeline(pipeline);
        vertex_sys.begin_pass();

        let mut i = 0usize;
        while i < collector.outline_items.len() {
            let batch_start = i;
            let geom_id = collector.outline_items[i].0;

            while i < collector.outline_items.len() && collector.outline_items[i].0 == geom_id {
                i += 1;
            }
            let batch_count = (i - batch_start) as u32;

            if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
                if vertex_sys.bind(&mut render_pass, geom_id).is_ok() {
                    let first_instance = collector.outline_items[batch_start].1;
                    render_pass.draw_indexed(
                        0..index_info.count,
                        0,
                        first_instance..(first_instance + batch_count),
                    );
                }
            }
        }
    }
}
