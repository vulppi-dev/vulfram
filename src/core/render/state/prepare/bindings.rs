use super::super::RenderState;
use crate::core::resources::{FrameComponent, CameraComponent};

impl RenderState {
    pub(crate) fn update_bind_groups(&mut self, device: &wgpu::Device, with_shadows: bool) {
        let bindings = self.bindings.as_mut().unwrap();
        let library = self.library.as_ref().unwrap();
        let light_system = self.light_system.as_ref().unwrap();
        let shadow_manager = self.shadow.as_ref().unwrap();

        let forward_atlas_view = match self.forward_atlas.as_ref() {
            Some(atlas) => atlas.view(),
            None => &library.fallback_forward_atlas_view,
        };

        let shadow_atlas_view = if with_shadows {
            shadow_manager.atlas.view()
        } else {
            &library.fallback_shadow_view
        };

        if bindings.shared_group.is_none() {
            bindings.shared_group = Some(
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!(
                        "BindGroup Shared (Consolidated, shadows={})",
                        with_shadows
                    )),
                    layout: &library.layout_shared,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.frame_pool.buffer(),
                                offset: 0,
                                size: Some(
                                    std::num::NonZeroU64::new(
                                        std::mem::size_of::<FrameComponent>() as u64,
                                    )
                                    .unwrap(),
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.camera_pool.buffer(),
                                offset: 0,
                                size: Some(
                                    std::num::NonZeroU64::new(
                                        std::mem::size_of::<CameraComponent>() as u64,
                                    )
                                    .unwrap(),
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: light_system.light_params.buffer(),
                                offset: 0,
                                size: Some(
                                    std::num::NonZeroU64::new(
                                        std::mem::size_of::<super::super::light::LightDrawParams>() as u64,
                                    )
                                    .unwrap(),
                                ),
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: light_system.lights.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: light_system.visible_indices.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: light_system.visible_counts.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: shadow_manager.params_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::TextureView(shadow_atlas_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: shadow_manager.page_table.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: shadow_manager.point_light_vp.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                        },
                        wgpu::BindGroupEntry {
                            binding: 11,
                            resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                        },
                        wgpu::BindGroupEntry {
                            binding: 12,
                            resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                        },
                        wgpu::BindGroupEntry {
                            binding: 13,
                            resource: wgpu::BindingResource::Sampler(
                                &library.samplers.linear_repeat,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 14,
                            resource: wgpu::BindingResource::Sampler(&library.samplers.comparison),
                        },
                        wgpu::BindGroupEntry {
                            binding: 15,
                            resource: wgpu::BindingResource::TextureView(forward_atlas_view),
                        },
                    ],
                }),
            );
        }

        // 3. Create Model Bind Group (Group 1)
        if bindings.model_bind_group.is_none() {
            bindings.model_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BindGroup Object (Dynamic Instance Data)"),
                layout: &library.layout_object,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.instance_pool.buffer(),
                        offset: 0,
                        size: None,
                    }),
                }],
            }));
        }

        // 4. Create Shadow Model Bind Group (Group 1 for shadow pass)
        if bindings.shadow_model_bind_group.is_none() {
            bindings.shadow_model_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Shadow Object (Dynamic Instance Data)"),
                    layout: &library.layout_object,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: bindings.shadow_instance_pool.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    }],
                }));
        }
    }
}
