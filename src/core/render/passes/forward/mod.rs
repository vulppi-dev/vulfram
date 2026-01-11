mod branches;
mod collector;
mod draw;

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};

pub fn pass_forward(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let scene = &render_state.scene;

    // Split borrows
    let (vertex_sys, bindings, library, light_system, collector, cache, gizmos) = (
        render_state.vertex.as_mut().unwrap(),
        render_state.bindings.as_mut().unwrap(),
        render_state.library.as_ref().unwrap(),
        render_state.light_system.as_mut().unwrap(),
        &mut render_state.collector,
        &mut render_state.cache,
        &mut render_state.gizmos,
    );

    // 0. Ensure Depth Target exists and matches size (Lazy)
    if render_state.forward_depth_target.is_none() {
        if let Some((_, camera)) = scene.cameras.iter().next() {
            if let Some(target) = &camera.render_target {
                let size = target._texture.size();
                render_state.forward_depth_target = Some(crate::core::resources::RenderTarget::new(
                    device,
                    size,
                    wgpu::TextureFormat::Depth32Float,
                ));
            }
        }
    }

    let depth_target = render_state.forward_depth_target.as_ref();
    gizmos.prepare(device, queue);

    // Pre-cache Gizmo Pipeline once per pass if needed
    let gizmo_pipeline_key = if !gizmos.is_empty() {
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
        collector.clear();

        // 3. Collection & Sorting
        collector::collect_objects(scene, collector, camera_record, vertex_sys);

        // 4. Begin render pass
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

            // 5. Bind Shared (Group 0: Frame + Camera + ModelPool)
            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(*camera_id) as u32;
                let light_offset = light_system.draw_params_offset(camera_index as u32) as u32;
                render_pass.set_bind_group(0, shared_group, &[camera_offset, light_offset]);
            }

            // Write instances
            if !collector.instance_data.is_empty() {
                bindings
                    .instance_pool
                    .write_slice(0, &collector.instance_data);
            }

            // 6. Draw Batches
            draw::draw_batches(
                &mut render_pass,
                scene,
                library,
                collector,
                bindings,
                vertex_sys,
                frame_index,
                device,
                cache,
            );

            // 7. Draw Gizmos
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
                                        offset: 16,
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
                gizmos.draw(&mut render_pass);
            }
        }
    }
}
