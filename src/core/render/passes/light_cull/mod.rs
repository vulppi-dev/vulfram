use bytemuck::bytes_of;

use crate::core::render::RenderState;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightCullParams {
    light_count: u32,
    camera_count: u32,
    max_lights_per_camera: u32,
    _padding: u32,
}

pub fn pass_light_cull(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    _frame_index: u64,
) {
    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let light_system = match render_state.light_system.as_mut() {
        Some(sys) => sys,
        None => return,
    };

    let light_count = light_system.light_count as u32;
    let camera_count = render_state.scene.cameras.len() as u32;

    if light_count == 0 || camera_count == 0 {
        render_state.passes.light_cull.bind_group = None;
        return;
    }

    light_system.camera_count = camera_count as u32;
    light_system.max_lights_per_camera = light_count;

    let params = LightCullParams {
        light_count,
        camera_count,
        max_lights_per_camera: light_count,
        _padding: 0,
    };

    if let Some(buffer) = light_system.params_buffer.as_ref() {
        light_system
            .queue
            .write_buffer(buffer, 0, bytes_of(&params));
    }

    if render_state.passes.light_cull.pipeline.is_none() {
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Light Cull Pipeline"),
            layout: Some(&library.light_cull_pipeline_layout),
            module: &library.light_cull_shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        render_state.passes.light_cull.pipeline = Some(pipeline);
    }

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Light Cull Bind Group"),
        layout: &library.layout_light_cull,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: light_system.lights.buffer(),
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: light_system.visible_indices.buffer(),
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: light_system.visible_counts.buffer(),
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: light_system
                        .params_buffer
                        .as_ref()
                        .expect("Light cull params buffer missing"),
                    offset: 0,
                    size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<LightCullParams>() as u64)
                            .unwrap(),
                    ),
                }),
            },
        ],
    });

    render_state.passes.light_cull.bind_group = Some(bind_group);

    if let Some(pipeline) = render_state.passes.light_cull.pipeline.as_ref() {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Light Cull Pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(pipeline);
        if let Some(group) = render_state.passes.light_cull.bind_group.as_ref() {
            cpass.set_bind_group(0, group, &[]);
        }

        let workgroup_size = 64u32;
        let dispatch_count = (light_count + workgroup_size - 1) / workgroup_size;
        cpass.dispatch_workgroups(dispatch_count, 1, 1);
    }
}
