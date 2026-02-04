use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::render::passes::update_post_uniform_buffer;
use crate::core::render::state::ResourceLibrary;
use crate::core::ui::state::UiState;
use crate::core::ui::types::UiRenderTarget;

fn build_compose_bind_group(
    device: &wgpu::Device,
    library: &ResourceLibrary,
    target_view: &wgpu::TextureView,
    outline_view: &wgpu::TextureView,
    ssao_view: &wgpu::TextureView,
    bloom_view: &wgpu::TextureView,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compose Bind Group"),
        layout: &library.layout_target,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(target_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
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

pub fn pass_compose(
    render_state: &mut RenderState,
    ui_state: &UiState,
    window_id: u32,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface_texture: &wgpu::SurfaceTexture,
    config: &wgpu::SurfaceConfiguration,
    frame_index: u64,
) {
    let view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // 2. Get or Create Compose Pipeline
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let post_config = render_state.environment.post.clone();
    let uniform_buffer = match render_state.post_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };
    update_post_uniform_buffer(&post_config, uniform_buffer, queue, frame_index);

    let key = PipelineKey {
        shader_id: ShaderId::Compose as u64,
        color_format: config.format,
        color_target_count: 1,
        depth_format: None,
        sample_count: 1,
        topology: wgpu::PrimitiveTopology::TriangleList,
        cull_mode: None,
        front_face: wgpu::FrontFace::Ccw,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        blend: None,
    };

    let mut items = build_compose_items(render_state, ui_state, window_id, config);

    let pipeline = {
        let cache = &mut render_state.cache;
        cache.get_or_create(key, frame_index, || {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compose Pipeline Layout"),
                bind_group_layouts: &[&library.layout_target],
                ..Default::default()
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Compose Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &library.compose_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &library.compose_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
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
        })
    };

    // 3. Begin compose pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Compose Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
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

    for item in items.drain(..) {
        render_pass.set_viewport(
            item.viewport.x,
            item.viewport.y,
            item.viewport.w,
            item.viewport.h,
            0.0,
            1.0,
        );

        // 5. Create Bind Group for this camera's target
        let bind_group = build_compose_bind_group(
            device,
            library,
            &item.target_view,
            &item.outline_view,
            &item.ssao_view,
            &item.bloom_view,
            uniform_buffer,
        );

        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

struct ComposeViewport {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

struct ComposeItem {
    layer: i32,
    order: i32,
    stable: usize,
    target_view: wgpu::TextureView,
    outline_view: wgpu::TextureView,
    ssao_view: wgpu::TextureView,
    bloom_view: wgpu::TextureView,
    viewport: ComposeViewport,
}

fn build_compose_items(
    render_state: &RenderState,
    ui_state: &UiState,
    window_id: u32,
    config: &wgpu::SurfaceConfiguration,
) -> Vec<ComposeItem> {
    let library = match render_state.library.as_ref() {
        Some(library) => library,
        None => return Vec::new(),
    };
    let mut items = Vec::new();
    let mut stable = 0usize;

    for (_id, record) in &render_state.scene.cameras {
        if record.target_texture_id.is_some() {
            continue;
        }
        let target = match record
            .post_target
            .as_ref()
            .or(record.render_target.as_ref())
        {
            Some(t) => t,
            None => continue,
        };
        let outline_view = record
            .outline_target
            .as_ref()
            .map(|target| target.view.clone())
            .unwrap_or_else(|| library.fallback_view.clone());
        let ssao_view = record
            .ssao_blur_target
            .as_ref()
            .map(|target| target.view.clone())
            .unwrap_or_else(|| library.fallback_view.clone());
        let bloom_view = record
            .bloom_target
            .as_ref()
            .map(|target| target.view.clone())
            .unwrap_or_else(|| library.fallback_view.clone());

        let (x, y) = record
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_position(config.width, config.height))
            .unwrap_or((0, 0));
        let (width, height) = record
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_size(config.width, config.height))
            .unwrap_or((config.width, config.height));

        items.push(ComposeItem {
            layer: record.layer,
            order: record.order,
            stable,
            target_view: target.view.clone(),
            outline_view,
            ssao_view,
            bloom_view,
            viewport: ComposeViewport {
                x: x as f32,
                y: y as f32,
                w: width as f32,
                h: height as f32,
            },
        });
        stable += 1;
    }

    for (context_id, context) in &ui_state.contexts {
        if context.window_id != window_id {
            continue;
        }
        if context.target != UiRenderTarget::Screen {
            continue;
        }
        let Some(target) = context.render_target.as_ref() else {
            log::warn!("Ui context {:?} missing render target", context_id);
            continue;
        };
        items.push(ComposeItem {
            layer: context.z_index,
            order: 0,
            stable,
            target_view: target.view.clone(),
            outline_view: library.fallback_view.clone(),
            ssao_view: library.fallback_view.clone(),
            bloom_view: library.fallback_view.clone(),
            viewport: ComposeViewport {
                x: context.screen_rect.x,
                y: context.screen_rect.y,
                w: context.screen_rect.w,
                h: context.screen_rect.h,
            },
        });
        stable += 1;
    }

    items.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then(a.order.cmp(&b.order))
            .then(a.stable.cmp(&b.stable))
    });
    items
}
