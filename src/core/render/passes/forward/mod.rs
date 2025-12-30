use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::resources::VertexStream;

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

    let depth_target = render_state.passes.forward.depth_target.as_ref();

    // 1. Sort cameras by order

    let mut sorted_cameras: Vec<_> = scene.cameras.iter().collect();

    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (camera_id, camera_record) in sorted_cameras {
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
                let offset = bindings.camera_pool.get_offset(*camera_id) as u32;

                render_pass.set_bind_group(0, shared_group, &[offset]);
            }

            // 5. Filter and draw models

            for (model_id, model_record) in &scene.models {
                // Check layer mask

                if (model_record.layer_mask & camera_record.layer_mask) == 0 {
                    continue;
                }

                // Bind Object (Group 1: Model)

                if let Some(object_group) = bindings.object_group.as_ref() {
                    let offset = bindings.model_pool.get_offset(*model_id) as u32;

                    render_pass.set_bind_group(1, object_group, &[offset]);
                }

                // Bind Geometry

                if let Ok(Some(index_info)) = vertex_sys.index_info(model_record.geometry_id) {
                    let _ = vertex_sys.bind(&mut render_pass, model_record.geometry_id);

                    // Pipeline Cache

                    let key = PipelineKey {
                        shader_id: 0, // Standard Forward Shader

                        color_format: wgpu::TextureFormat::Rgba32Float,

                        depth_format: Some(wgpu::TextureFormat::Depth24Plus),

                        sample_count: 1,

                        topology: wgpu::PrimitiveTopology::TriangleList,

                        cull_mode: Some(wgpu::Face::Back),

                        front_face: wgpu::FrontFace::Ccw,

                        depth_write_enabled: true,

                        depth_compare: wgpu::CompareFunction::Less,

                        blend: None,
                    };

                    let pipeline = cache.get_or_create(key, frame_index, || {
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Forward Standard Pipeline"),

                            layout: Some(&library.forward_pipeline_layout),

                            vertex: wgpu::VertexState {
                                module: &library.forward_shader,

                                entry_point: Some("vs_main"),

                                buffers: &[
                                    // 0: Position
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::Position.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,

                                            offset: 0,

                                            shader_location: 0,
                                        }],
                                    },
                                    // 1: Normal
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::Normal.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x3,

                                            offset: 0,

                                            shader_location: 1,
                                        }],
                                    },
                                    // 2: Tangent
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::Tangent.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x4,

                                            offset: 0,

                                            shader_location: 2,
                                        }],
                                    },
                                    // 3: Color0
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::Color0.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x4,

                                            offset: 0,

                                            shader_location: 3,
                                        }],
                                    },
                                    // 4: UV0
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::UV0.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x2,

                                            offset: 0,

                                            shader_location: 4,
                                        }],
                                    },
                                    // 5: UV1
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::UV1.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Float32x2,

                                            offset: 0,

                                            shader_location: 5,
                                        }],
                                    },
                                    // 6: Joints
                                    wgpu::VertexBufferLayout {
                                        array_stride: VertexStream::Joints.stride_bytes(),

                                        step_mode: wgpu::VertexStepMode::Vertex,

                                        attributes: &[wgpu::VertexAttribute {
                                            format: wgpu::VertexFormat::Uint16x4,

                                            offset: 0,

                                            shader_location: 6,
                                        }],
                                    },
                                    // 7: Weights
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
                                module: &library.forward_shader,

                                entry_point: Some("fs_main"),

                                targets: &[Some(wgpu::ColorTargetState {
                                    format: key.color_format,

                                    blend: key.blend,

                                    write_mask: wgpu::ColorWrites::ALL,
                                })],

                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            }),

                            primitive: wgpu::PrimitiveState {
                                topology: key.topology,

                                strip_index_format: None,

                                front_face: key.front_face,

                                cull_mode: key.cull_mode,

                                unclipped_depth: false,

                                polygon_mode: wgpu::PolygonMode::Fill,

                                conservative: false,
                            },

                            depth_stencil: key.depth_format.map(|format| wgpu::DepthStencilState {
                                format,

                                depth_write_enabled: key.depth_write_enabled,

                                depth_compare: key.depth_compare,

                                stencil: wgpu::StencilState::default(),

                                bias: wgpu::DepthBiasState::default(),
                            }),

                            multisample: wgpu::MultisampleState {
                                count: key.sample_count,

                                mask: !0,

                                alpha_to_coverage_enabled: false,
                            },

                            multiview: None,

                            cache: None,
                        })
                    });

                    render_pass.set_pipeline(pipeline);

                    render_pass.draw_indexed(0..index_info.count, 0, 0..1);
                }
            }
        }
    }
}
