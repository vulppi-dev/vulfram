use bytemuck::{Pod, Zeroable};

use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SsaoUniform {
    proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    params0: [f32; 4],
    params1: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SsaoBlurUniform {
    params0: [f32; 4],
}

fn update_ssao_uniform(
    record: &crate::core::resources::CameraRecord,
    config: &crate::core::resources::PostProcessConfig,
    buffer: &wgpu::Buffer,
    queue: &wgpu::Queue,
    frame_index: u64,
) {
    let proj = record.data.projection;
    let inv_proj = proj.inverse();
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

    let uniform = SsaoUniform {
        proj: proj.to_cols_array_2d(),
        inv_proj: inv_proj.to_cols_array_2d(),
        params0: [
            config.ssao_radius.max(0.001),
            config.ssao_bias.max(0.0),
            config.ssao_power.max(0.1),
            0.0,
        ],
        params1: [texel_x, texel_y, 0.0, frame_index as f32],
    };

    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&uniform));
}

fn update_ssao_blur_uniform(
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

    let uniform = SsaoBlurUniform {
        params0: [
            texel_x,
            texel_y,
            config.ssao_blur_depth_threshold.max(0.0001),
            config.ssao_blur_radius.clamp(0.0, 6.0),
        ],
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
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
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

pub fn pass_ssao(
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
    let depth_target = match render_state.forward_depth_target.as_ref() {
        Some(target) => target,
        None => return,
    };
    let use_msaa = depth_target.sample_count > 1;
    let ssao_buffer = match render_state.ssao_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (_id, record) in sorted_cameras {
        let target = match &record.ssao_target {
            Some(t) => t,
            None => continue,
        };

        if !post_config.ssao_enabled {
            clear_target(encoder, target, "SSAO Clear Pass");
            continue;
        }

        update_ssao_uniform(record, &post_config, ssao_buffer, queue, frame_index);

        let (pipeline, bind_group) = if use_msaa {
            let key = PipelineKey {
                shader_id: ShaderId::SsaoMsaa as u64,
                color_format: target.format,
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
                    label: Some("SSAO MSAA Pipeline"),
                    layout: Some(&library.ssao_msaa_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.ssao_msaa_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.ssao_msaa_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: target.format,
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
                label: Some("SSAO MSAA Bind Group"),
                layout: &library.layout_ssao_msaa,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ssao_buffer.as_entire_binding(),
                    },
                ],
            });

            (pipeline, bind_group)
        } else {
            let key = PipelineKey {
                shader_id: ShaderId::Ssao as u64,
                color_format: target.format,
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
                    label: Some("SSAO Pipeline"),
                    layout: Some(&library.ssao_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.ssao_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.ssao_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: target.format,
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
                label: Some("SSAO Bind Group"),
                layout: &library.layout_ssao,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ssao_buffer.as_entire_binding(),
                    },
                ],
            });

            (pipeline, bind_group)
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let size = target._texture.size();
        render_pass.set_pipeline(pipeline);
        render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub fn pass_ssao_blur(
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
    let depth_target = match render_state.forward_depth_target.as_ref() {
        Some(target) => target,
        None => return,
    };
    let use_msaa = depth_target.sample_count > 1;
    let blur_buffer = match render_state.ssao_blur_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (_id, record) in sorted_cameras {
        let input_target = match &record.ssao_target {
            Some(t) => t,
            None => continue,
        };
        let output_target = match &record.ssao_blur_target {
            Some(t) => t,
            None => continue,
        };

        if !post_config.ssao_enabled {
            clear_target(encoder, output_target, "SSAO Blur Clear Pass");
            continue;
        }

        update_ssao_blur_uniform(record, &post_config, blur_buffer, queue);

        let (pipeline, bind_group) = if use_msaa {
            let key = PipelineKey {
                shader_id: ShaderId::SsaoBlurMsaa as u64,
                color_format: output_target.format,
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
                    label: Some("SSAO Blur MSAA Pipeline"),
                    layout: Some(&library.ssao_blur_msaa_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.ssao_blur_msaa_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.ssao_blur_msaa_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: output_target.format,
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
                label: Some("SSAO Blur MSAA Bind Group"),
                layout: &library.layout_ssao_blur_msaa,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&depth_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: blur_buffer.as_entire_binding(),
                    },
                ],
            });

            (pipeline, bind_group)
        } else {
            let key = PipelineKey {
                shader_id: ShaderId::SsaoBlur as u64,
                color_format: output_target.format,
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
                    label: Some("SSAO Blur Pipeline"),
                    layout: Some(&library.ssao_blur_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &library.ssao_blur_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &library.ssao_blur_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: output_target.format,
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
                label: Some("SSAO Blur Bind Group"),
                layout: &library.layout_ssao_blur,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&depth_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: blur_buffer.as_entire_binding(),
                    },
                ],
            });

            (pipeline, bind_group)
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Blur Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let size = output_target._texture.size();
        render_pass.set_pipeline(pipeline);
        render_pass.set_viewport(0.0, 0.0, size.width as f32, size.height as f32, 0.0, 1.0);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
