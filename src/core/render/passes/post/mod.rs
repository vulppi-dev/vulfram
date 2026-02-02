use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::PostProcessConfig;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostProcessUniform {
    params0: [f32; 4],
    params1: [f32; 4],
    params2: [f32; 4],
    params3: [f32; 4],
    params4: [f32; 4],
    params5: [f32; 4],
}

impl PostProcessUniform {
    fn from_config(config: &PostProcessConfig, frame_index: u64) -> Self {
        let mut flags = 0u32;
        if config.filter_enabled {
            flags |= 1;
        }
        if config.cell_shading {
            flags |= 1 << 1;
        }
        if config.outline_enabled {
            flags |= 1 << 2;
        }
        if config.ssao_enabled {
            flags |= 1 << 3;
        }
        if config.bloom_enabled {
            flags |= 1 << 4;
        }

        let outline_threshold = config.outline_threshold.clamp(0.0, 0.999);
        let outline_quality = config.outline_quality.clamp(0.0, 1.0);

        Self {
            params0: [
                config.filter_exposure,
                config.filter_gamma.max(0.001),
                config.filter_saturation,
                config.filter_contrast,
            ],
            params1: [
                config.filter_vignette,
                config.filter_grain,
                config.filter_chromatic_aberration,
                config.filter_blur,
            ],
            params2: [
                config.outline_strength,
                outline_threshold,
                config.filter_posterize_steps,
                flags as f32,
            ],
            params3: [
                frame_index as f32,
                config.filter_sharpen,
                config.outline_width,
                outline_quality,
            ],
            params4: [config.ssao_strength, config.ssao_power, 0.0, 0.0],
            params5: [
                config.bloom_threshold,
                config.bloom_knee,
                config.bloom_intensity,
                config.bloom_scatter,
            ],
        }
    }
}

pub fn update_post_uniform_buffer(
    config: &PostProcessConfig,
    buffer: &wgpu::Buffer,
    queue: &wgpu::Queue,
    frame_index: u64,
) {
    let uniform = PostProcessUniform::from_config(config, frame_index);
    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&uniform));
}

fn build_post_bind_group(
    device: &wgpu::Device,
    library: &crate::core::render::state::ResourceLibrary,
    target_view: &wgpu::TextureView,
    outline_view: &wgpu::TextureView,
    ssao_view: &wgpu::TextureView,
    bloom_view: &wgpu::TextureView,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Post Bind Group"),
        layout: &library.layout_target,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(target_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(outline_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(ssao_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(bloom_view),
            },
        ],
    })
}

pub fn pass_post(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let cache = &mut render_state.cache;

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    let post_config = render_state.environment.post.clone();
    let uniform_buffer = match render_state.post_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };
    update_post_uniform_buffer(&post_config, uniform_buffer, queue, frame_index);

    for (_id, record) in sorted_cameras {
        let input_target = match &record.render_target {
            Some(t) => t,
            None => continue,
        };
        let output_target = match &record.post_target {
            Some(t) => t,
            None => continue,
        };

        let size = input_target._texture.size();
        let width = size.width;
        let height = size.height;

        let key = PipelineKey {
            shader_id: ShaderId::Post as u64,
            color_format: output_target.format,
            color_target_count: 1,
            depth_format: None,
            sample_count: output_target.sample_count,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            blend: None,
        };

        let pipeline = cache.get_or_create(key, frame_index, || {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Post Pipeline Layout"),
                bind_group_layouts: &[&library.layout_target],
                ..Default::default()
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Post Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &library.post_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &library.post_shader,
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

        let outline_view = record
            .outline_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let ssao_view = record
            .ssao_blur_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let bloom_view = record
            .bloom_target
            .as_ref()
            .map(|target| &target.view)
            .unwrap_or(&library.fallback_view);
        let bind_group = build_post_bind_group(
            device,
            library,
            &input_target.view,
            outline_view,
            ssao_view,
            bloom_view,
            uniform_buffer,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_target.view,
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

        render_pass.set_pipeline(pipeline);

        render_pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
