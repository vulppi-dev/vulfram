use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::resources::{CameraComponent, VertexStream, wgpu_projection_correction};
use glam::Vec4Swizzles;

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
    if !shadow_manager.is_dirty {
        return;
    }

    // If the manager is dirty, it means something in the scene changed (light or model).
    // We must ensure all currently used pages are re-rendered.
    for record in shadow_manager.cache.values_mut() {
        record.is_dirty = true;
    }

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

    // Collect pages (re-render on dirty scenes)
    let mut pages_to_render = Vec::new();

    let mut light_ids: Vec<u32> = render_state.scene.lights.keys().copied().collect();
    light_ids.sort_unstable();
    for (light_index, light_id) in light_ids.iter().copied().enumerate() {
        let light_record = match render_state.scene.lights.get(&light_id) {
            Some(record) => record,
            None => continue,
        };

        if !light_record.cast_shadow {
            continue;
        }

        let mut light_views = Vec::new();
        let mut light_projs = Vec::new();

        match light_record.data.kind_flags.x {
            1 => {
                // Point Light (6 faces)
                let pos = light_record.data.position.xyz();
                let range = light_record.data.intensity_range.y;
                let projection = wgpu_projection_correction()
                    * glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, range);

                // +X, -X, +Y, -Y, +Z, -Z
                let targets = [
                    (glam::Vec3::X, glam::Vec3::Y),
                    (glam::Vec3::NEG_X, glam::Vec3::Y),
                    (glam::Vec3::Y, glam::Vec3::NEG_Z),
                    (glam::Vec3::NEG_Y, glam::Vec3::Z),
                    (glam::Vec3::Z, glam::Vec3::Y),
                    (glam::Vec3::NEG_Z, glam::Vec3::Y),
                ];

                for (target, up) in targets {
                    light_views.push(glam::Mat4::look_to_rh(pos, target, up));
                    light_projs.push(projection);
                }
            }
            _ => {
                // Directional or Spot
                light_views.push(light_record.data.view);
                light_projs.push(light_record.data.projection);
            }
        }

        for (face_index, (light_view, light_proj)) in light_views
            .into_iter()
            .zip(light_projs.into_iter())
            .enumerate()
        {
            let light_view_proj = light_proj * light_view;
            let required =
                shadow_manager.identify_required_pages(light_view_proj, camera_inv_view_proj);

            for (x, y) in required {
                let shadow_light_id = light_index as u32;
                if let Some(handle) = shadow_manager.request_page(
                    shadow_light_id,
                    face_index as u32,
                    x,
                    y,
                    frame_index,
                ) {
                    let key = crate::core::render::shadow::ShadowPageKey {
                        light_id: shadow_light_id,
                        face: face_index as u32,
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
    }

    if pages_to_render.is_empty() {
        return;
    }

    struct PageRender {
        layer: u32,
        key: crate::core::render::shadow::ShadowPageKey,
        shadow_cam_id: u32,
        transform: (f32, f32, f32, f32, u32),
    }

    let info = shadow_manager.atlas.info();
    let mut render_pages = Vec::new();

    for (key, handle, light_view, light_proj) in pages_to_render {
        let page_vp = shadow_manager.get_page_view_projection(light_view, light_proj, key.x, key.y);

        let shadow_camera_data = CameraComponent {
            view_projection: page_vp,
            ..Default::default()
        };

        // Use a very low ID for shadow cameras to fit in the initial pool capacity (128)
        let shadow_cam_id = 10 + key.light_id * 6 + key.face;
        bindings
            .camera_pool
            .write(shadow_cam_id, &shadow_camera_data);

        let transform = match shadow_manager.atlas.get_uv_transform(handle) {
            Some(transform) => transform,
            None => continue,
        };

        render_pages.push(PageRender {
            layer: transform.4,
            key,
            shadow_cam_id,
            transform,
        });
    }

    render_pages.sort_by_key(|page| page.layer);

    let mut i = 0;
    while i < render_pages.len() {
        let layer = render_pages[i].layer;
        let atlas_layer_view = shadow_manager
            .atlas
            .layer_view(layer)
            .expect("Atlas layer view missing");

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Shadow Page Update Pass - Layer {}", layer)),
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

        while i < render_pages.len() && render_pages[i].layer == layer {
            let page = &render_pages[i];
            let vx = page.transform.2 * (info.tiles_w * info.pitch_px) as f32;
            let vy = page.transform.3 * (info.tiles_h * info.pitch_px) as f32;
            let vw = page.transform.0 * (info.tiles_w * info.pitch_px) as f32;
            let vh = page.transform.1 * (info.tiles_h * info.pitch_px) as f32;

            rpass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            rpass.set_scissor_rect(vx as u32, vy as u32, vw as u32, vh as u32);

            if let Some(shared_group) = bindings.shared_group.as_ref() {
                let camera_offset = bindings.camera_pool.get_offset(page.shadow_cam_id) as u32;
                rpass.set_bind_group(0, shared_group, &[camera_offset, 0]);
            }

            for (model_id, model_record) in &render_state.scene.models {
                if !model_record.cast_shadow {
                    continue;
                }

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
                                // Bias to reduce self-shadowing artifacts (shadow acne).
                                bias: wgpu::DepthBiasState {
                                    constant: 2,
                                    slope_scale: 2.0,
                                    clamp: 0.0,
                                },
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

            i += 1;
        }
    }

    shadow_manager.clear_dirty();
}
