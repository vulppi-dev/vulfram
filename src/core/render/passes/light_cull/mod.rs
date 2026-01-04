use bytemuck::bytes_of;

use crate::core::render::RenderState;
use crate::core::render::cache::{ComputePipelineKey, ShaderId};
use crate::core::render::state::{FrustumPlane, LightCullingSystem, ResourceLibrary};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightCullParams {
    light_count: u32,
    camera_count: u32,
    max_lights_per_camera: u32,
    _padding: u32,
}

const LIGHT_CULL_WORKGROUP_SIZE: u32 = 64;
const PLANES_PER_CAMERA: u32 = 6;

fn normalize_plane(plane: glam::Vec4) -> glam::Vec4 {
    let normal = plane.truncate();
    let len = normal.length();
    if len > 0.0 {
        plane / len
    } else {
        plane
    }
}

fn extract_frustum_planes(view_projection: glam::Mat4) -> [FrustumPlane; 6] {
    let m = view_projection.to_cols_array_2d();
    let row0 = glam::Vec4::new(m[0][0], m[1][0], m[2][0], m[3][0]);
    let row1 = glam::Vec4::new(m[0][1], m[1][1], m[2][1], m[3][1]);
    let row2 = glam::Vec4::new(m[0][2], m[1][2], m[2][2], m[3][2]);
    let row3 = glam::Vec4::new(m[0][3], m[1][3], m[2][3], m[3][3]);

    [
        FrustumPlane {
            data: normalize_plane(row3 + row0),
        }, // left
        FrustumPlane {
            data: normalize_plane(row3 - row0),
        }, // right
        FrustumPlane {
            data: normalize_plane(row3 + row1),
        }, // bottom
        FrustumPlane {
            data: normalize_plane(row3 - row1),
        }, // top
        FrustumPlane {
            data: normalize_plane(row2),
        }, // near (WebGPU z 0..1)
        FrustumPlane {
            data: normalize_plane(row3 - row2),
        }, // far
    ]
}

fn build_light_cull_bind_group(
    device: &wgpu::Device,
    library: &ResourceLibrary,
    light_system: &LightCullingSystem,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: light_system.camera_frustums.buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        ],
    })
}
pub fn pass_light_cull(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
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
    let mut sorted_cameras: Vec<_> = render_state.scene.cameras.iter().collect();
    sorted_cameras.sort_by_key(|(_, record)| record.order);
    let camera_count = sorted_cameras.len() as u32;

    light_system.camera_count = camera_count as u32;
    light_system.max_lights_per_camera = light_count;

    if light_count == 0 || camera_count == 0 {
        light_system.bind_group = None;
        return;
    }

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

    if camera_count > 0 {
        let mut planes = Vec::with_capacity((camera_count * PLANES_PER_CAMERA) as usize);
        for (_id, record) in &sorted_cameras {
            planes.extend_from_slice(&extract_frustum_planes(record.data.view_projection));
        }
        light_system.camera_frustums.write_slice(0, &planes);
    }

    let cache = &mut render_state.cache;
    let key = ComputePipelineKey {
        shader_id: ShaderId::LightCull as u64,
    };
    let pipeline = cache.get_or_create_compute(key, frame_index, || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Light Cull Pipeline"),
            layout: Some(&library.light_cull_pipeline_layout),
            module: &library.light_cull_shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    });

    let bind_group = build_light_cull_bind_group(device, library, light_system);

    light_system.bind_group = Some(bind_group);

    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Light Cull Pass"),
        timestamp_writes: None,
    });
    cpass.set_pipeline(pipeline);
    if let Some(group) = light_system.bind_group.as_ref() {
        cpass.set_bind_group(0, group, &[]);
    }

    let dispatch_count = (light_count + LIGHT_CULL_WORKGROUP_SIZE - 1) / LIGHT_CULL_WORKGROUP_SIZE;
    cpass.dispatch_workgroups(dispatch_count, 1, 1);
}
