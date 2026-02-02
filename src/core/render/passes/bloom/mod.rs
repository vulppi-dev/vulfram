use bytemuck::{Pod, Zeroable};

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};

const BLOOM_DOWNSAMPLE_COUNT: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BloomUniform {
    params0: [f32; 4],
    params1: [f32; 4],
}

fn update_bloom_uniform(
    record: &crate::core::resources::CameraRecord,
    config: &crate::core::resources::PostProcessConfig,
    buffer: &wgpu::Buffer,
    queue: &wgpu::Queue,
) {
    let size = record
        .render_target
        .as_ref()
        .map(|target| target._texture.size())
        .unwrap_or(wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        });
    let texel_x = 1.0 / size.width.max(1) as f32;
    let texel_y = 1.0 / size.height.max(1) as f32;

    let uniform = BloomUniform {
        params0: [config.bloom_threshold, config.bloom_knee, texel_x, texel_y],
        params1: [config.bloom_scatter, 0.0, 0.0, 0.0],
    };

    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&uniform));
}

fn clear_target(
    encoder: &mut wgpu::CommandEncoder,
    target: &crate::core::resources::RenderTarget,
    label: &str,
) {
    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &target.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

pub fn pass_bloom(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let post_config = render_state.environment.post.clone();
    let library = match render_state.library.as_ref() {
        Some(lib) => lib,
        None => return,
    };
    let bloom_buffer = match render_state.bloom_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (_id, record) in sorted_cameras {
        let input_target = match &record.render_target {
            Some(t) => t,
            None => continue,
        };
        let bloom_target = match &record.bloom_target {
            Some(t) => t,
            None => continue,
        };

        if !post_config.bloom_enabled {
            clear_target(encoder, bloom_target, "Bloom Clear");
            continue;
        }

        update_bloom_uniform(record, &post_config, bloom_buffer, queue);

        let mut chain_targets = Vec::with_capacity(BLOOM_DOWNSAMPLE_COUNT);
        for target in record.bloom_chain.iter() {
            if let Some(target) = target {
                chain_targets.push(target);
            }
        }

        if chain_targets.len() < BLOOM_DOWNSAMPLE_COUNT {
            clear_target(encoder, bloom_target, "Bloom Clear (Missing Chain)");
            continue;
        }

        // Prefilter
        {
            let key = PipelineKey {
                shader_id: ShaderId::BloomPrefilter as u64,
                color_format: chain_targets[0].format,
                depth_format: None,
                sample_count: 1,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: None,
            };

            let pipeline = render_state.cache.get_or_create(key, frame_index, || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Prefilter Pipeline"),
                    layout: Some(&library.bloom_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.bloom_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.bloom_shader,
                        entry_point: Some("fs_prefilter"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: chain_targets[0].format,
                            blend: None,
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Prefilter Bind Group"),
                layout: &library.layout_bloom,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bloom_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Prefilter Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &chain_targets[0].view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let size = chain_targets[0]._texture.size();
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Downsample
        for level in 1..BLOOM_DOWNSAMPLE_COUNT {
            let src = chain_targets[level - 1];
            let dst = chain_targets[level];

            let key = PipelineKey {
                shader_id: ShaderId::BloomDownsample as u64,
                color_format: dst.format,
                depth_format: None,
                sample_count: 1,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: None,
            };

            let pipeline = render_state.cache.get_or_create(key, frame_index, || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Downsample Pipeline"),
                    layout: Some(&library.bloom_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.bloom_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.bloom_shader,
                        entry_point: Some("fs_downsample"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: dst.format,
                            blend: None,
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Downsample Bind Group"),
                layout: &library.layout_bloom,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bloom_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Downsample Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let size = dst._texture.size();
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Upsample + combine
        for level in (1..BLOOM_DOWNSAMPLE_COUNT).rev() {
            let low = chain_targets[level];
            let high = chain_targets[level - 1];

            let key = PipelineKey {
                shader_id: ShaderId::BloomUpsample as u64,
                color_format: high.format,
                depth_format: None,
                sample_count: 1,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: None,
            };

            let pipeline = render_state.cache.get_or_create(key, frame_index, || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Upsample Pipeline"),
                    layout: Some(&library.bloom_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.bloom_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.bloom_shader,
                        entry_point: Some("fs_upsample"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: high.format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Upsample Bind Group"),
                layout: &library.layout_bloom,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&low.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bloom_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Upsample Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &high.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let size = high._texture.size();
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Copy final into bloom target
        {
            let key = PipelineKey {
                shader_id: ShaderId::BloomCombine as u64,
                color_format: bloom_target.format,
                depth_format: None,
                sample_count: 1,
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                front_face: wgpu::FrontFace::Ccw,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: None,
            };

            let pipeline = render_state.cache.get_or_create(key, frame_index, || {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Combine Pipeline"),
                    layout: Some(&library.bloom_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.bloom_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.bloom_shader,
                        entry_point: Some("fs_combine"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: bloom_target.format,
                            blend: None,
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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Combine Bind Group"),
                layout: &library.layout_bloom,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&chain_targets[0].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bloom_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Combine Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom_target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let size = bloom_target._texture.size();
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

pub fn bloom_chain_size(base: u32, level: usize) -> u32 {
    let divisor = 2u32.pow(level as u32 + 1);
    (base / divisor).max(1)
}
