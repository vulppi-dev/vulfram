use crate::core::render::RenderState;
use crate::core::render::cache::{PipelineKey, ShaderId};
use crate::core::resources::SkyboxMode;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SkyboxUniform {
    inv_view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    intensity: [f32; 4],
    ground_color: [f32; 4],
    horizon_color: [f32; 4],
    sky_color: [f32; 4],
    params: [f32; 4],
}

pub fn pass_skybox(
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

    let skybox = &render_state.environment.skybox;
    if matches!(skybox.mode, SkyboxMode::None) {
        return;
    }

    let sample_count = render_state.msaa_sample_count();

    if let Some((_, camera)) = render_state.scene.cameras.iter().next() {
        if let Some(target) = &camera.render_target {
            let size = target._texture.size();
            let needs_msaa = sample_count > 1
                && match render_state.forward_msaa_target.as_ref() {
                    Some(existing) => {
                        let existing_size = existing._texture.size();
                        existing_size.width != size.width
                            || existing_size.height != size.height
                            || existing.sample_count != sample_count
                    }
                    None => true,
                };

            if needs_msaa {
                render_state.forward_msaa_target =
                    Some(crate::core::resources::RenderTarget::new_with_samples(
                        device,
                        size,
                        wgpu::TextureFormat::Rgba16Float,
                        sample_count,
                    ));
            }
        }
    }

    let uniform_buffer = match render_state.skybox_uniform_buffer.as_ref() {
        Some(buffer) => buffer,
        None => return,
    };

    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);

    for (_camera_id, camera_record) in sorted_cameras {
        let target_view = match &camera_record.render_target {
            Some(target) => &target.view,
            None => continue,
        };

        let (color_view, resolve_target) = if sample_count > 1 {
            match render_state.forward_msaa_target.as_ref() {
                Some(msaa) => (&msaa.view, Some(target_view)),
                None => (target_view, None),
            }
        } else {
            (target_view, None)
        };

        let target_format = camera_record
            .render_target
            .as_ref()
            .map(|target| target.format)
            .unwrap_or(wgpu::TextureFormat::Rgba16Float);

        let pipeline_key = PipelineKey {
            shader_id: ShaderId::Skybox as u64,
            color_format: target_format,
            color_target_count: 1,
            depth_format: None,
            sample_count,
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            blend: None,
        };

        let pipeline = render_state.cache.get_or_create(pipeline_key, frame_index, || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Skybox Pipeline"),
                layout: Some(&library.skybox_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &library.skybox_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &library.skybox_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            })
        });

        let inv_view_proj = camera_record.data.view_projection.inverse();
        let camera_pos = camera_record.data.position.truncate();
        let mode_value = match skybox.mode {
            SkyboxMode::None => 0.0,
            SkyboxMode::Procedural => 1.0,
            SkyboxMode::Cubemap => 2.0,
        };

        let uniform = SkyboxUniform {
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            intensity: [skybox.intensity, 0.0, 0.0, 0.0],
            ground_color: [
                skybox.ground_color.x,
                skybox.ground_color.y,
                skybox.ground_color.z,
                1.0,
            ],
            horizon_color: [
                skybox.horizon_color.x,
                skybox.horizon_color.y,
                skybox.horizon_color.z,
                1.0,
            ],
            sky_color: [
                skybox.sky_color.x,
                skybox.sky_color.y,
                skybox.sky_color.z,
                1.0,
            ],
            params: [skybox.rotation, mode_value, 0.0, 0.0],
        };
        queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let skybox_view = match (skybox.mode, skybox.cubemap_texture_id) {
            (SkyboxMode::Cubemap, Some(id)) => render_state
                .scene
                .textures
                .get(&id)
                .map(|record| &record.view)
                .unwrap_or(&library.fallback_view),
            _ => &library.fallback_view,
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skybox Bind Group"),
            layout: &library.layout_skybox,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(skybox_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Skybox Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
