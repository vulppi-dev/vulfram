use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::resources::{CameraComponent, VertexStream};

pub fn pass_shadow_update(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let shadow_manager = match render_state.shadow.as_mut() {
        Some(s) => s,
        None => return,
    };

    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let bindings = match render_state.bindings.as_mut() {
        Some(b) => b,
        None => return,
    };

    let vertex_sys = match render_state.vertex.as_mut() {
        Some(v) => v,
        None => return,
    };

    let cache = &mut render_state.cache;

    // 1. Identify which pages need update for each light
    let primary_camera = match render_state.scene.cameras.values().next() {
        Some(c) => c,
        None => return,
    };

    let camera_inv_view_proj = primary_camera.data.view_projection.inverse();

    // Collect dirty pages
    let mut pages_to_render = Vec::new();

    let mut light_ids: Vec<u32> = render_state.scene.lights.keys().copied().collect();
    light_ids.sort_unstable();
    for (light_index, light_id) in light_ids.iter().copied().enumerate() {
        let light_record = match render_state.scene.lights.get(&light_id) {
            Some(record) => record,
            None => continue,
        };
        let light_view = light_record.data.view;
        let light_proj = light_record.data.projection;
        let light_view_proj = light_record.data.view_projection;

        let required =
            shadow_manager.identify_required_pages(light_view_proj, camera_inv_view_proj);

        for (x, y) in required {
            let shadow_light_id = light_index as u32;
            if let Some(handle) = shadow_manager.request_page(shadow_light_id, x, y, frame_index) {
                let key = crate::core::render::shadow::ShadowPageKey {
                    light_id: shadow_light_id,
                    x,
                    y,
                };
                if let Some(record) = shadow_manager.cache.get_mut(&key) {
                    pages_to_render.push((key, handle, light_view, light_proj));
                    record.is_dirty = false;
                }
            }
        }
    }

    if pages_to_render.is_empty() {
        return;
    }

    // 2. Render each dirty page
    for (key, handle, light_view, light_proj) in pages_to_render {
        let page_vp = shadow_manager.get_page_view_projection(light_view, light_proj, key.x, key.y);

        let shadow_camera_data = CameraComponent {
            view_projection: page_vp,
            ..Default::default()
        };

        // Use a very low ID for shadow cameras to fit in the initial pool capacity (128)
        let shadow_cam_id = 10 + key.light_id;
        bindings
            .camera_pool
            .write(shadow_cam_id, &shadow_camera_data);

        let info = shadow_manager.atlas.info();
        let transform = shadow_manager.atlas.get_uv_transform(handle).unwrap();

        let vx = transform.2 * (info.tiles_w * info.pitch_px) as f32;
        let vy = transform.3 * (info.tiles_h * info.pitch_px) as f32;
        let vw = transform.0 * (info.tiles_w * info.pitch_px) as f32;
        let vh = transform.1 * (info.tiles_h * info.pitch_px) as f32;

        let atlas_layer_view = shadow_manager
            .atlas
            .layer_view(transform.4)
            .expect("Atlas layer view missing");

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!(
                    "Shadow Page Update Pass - L{} P({},{})",
                    key.light_id, key.x, key.y
                )),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &atlas_layer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            rpass.set_scissor_rect(vx as u32, vy as u32, vw as u32, vh as u32);

            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(shadow_cam_id) as u32;
                rpass.set_bind_group(0, shared_group, &[camera_offset, 0]);
            }

            for (model_id, model_record) in &render_state.scene.models {
                if let Some(object_group) = bindings.object_group.as_ref() {
                    let offset = bindings.model_pool.get_offset(*model_id) as u32;
                    rpass.set_bind_group(1, object_group, &[offset]);
                }

                if let Ok(Some(index_info)) = vertex_sys.index_info(model_record.geometry_id) {
                    let _ = vertex_sys.bind(&mut rpass, model_record.geometry_id);

                    let key = PipelineKey {
                        shader_id: 2,                                      // Shadow Shader
                        color_format: wgpu::TextureFormat::Rgba8UnormSrgb, // Dummy, not used
                        depth_format: Some(wgpu::TextureFormat::Depth32Float),
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
                            label: Some("Shadow Pipeline"),
                            layout: Some(&library.shadow_pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &library.shadow_shader,
                                entry_point: Some("vs_main"),
                                buffers: &[wgpu::VertexBufferLayout {
                                    array_stride: VertexStream::Position.stride_bytes(),
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &[wgpu::VertexAttribute {
                                        format: wgpu::VertexFormat::Float32x3,
                                        offset: 0,
                                        shader_location: 0,
                                    }],
                                }],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            },
                            fragment: None, // Depth only
                            primitive: wgpu::PrimitiveState {
                                topology: key.topology,
                                front_face: key.front_face,
                                cull_mode: key.cull_mode,
                                ..Default::default()
                            },
                            depth_stencil: Some(wgpu::DepthStencilState {
                                format: wgpu::TextureFormat::Depth32Float,
                                depth_write_enabled: true,
                                depth_compare: wgpu::CompareFunction::Less,
                                stencil: wgpu::StencilState::default(),
                                bias: wgpu::DepthBiasState::default(),
                            }),
                            multisample: wgpu::MultisampleState::default(),
                            multiview: None,
                            cache: None,
                        })
                    });

                    rpass.set_pipeline(pipeline);
                    rpass.draw_indexed(0..index_info.count, 0, 0..1);
                }
            }
        }
    }
}
