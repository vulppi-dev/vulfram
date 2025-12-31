use std::collections::HashMap;

use crate::core::render::cache::RenderCache;
use crate::core::render::passes::RenderPasses;
use crate::core::render::shadow::ShadowManager;
use crate::core::resources::{
    CameraComponent, CameraRecord, FrameComponent, LightComponent, LightRecord, ModelComponent,
    ModelRecord, RenderTarget, StorageBufferPool, UniformBufferPool, VertexAllocatorConfig,
    VertexAllocatorSystem,
};

// -----------------------------------------------------------------------------
// Sub-systems
// -----------------------------------------------------------------------------

/// Collection of standard samplers for various rendering needs
pub struct SamplerSet {
    pub point_clamp: wgpu::Sampler,
    pub linear_clamp: wgpu::Sampler,
    pub point_repeat: wgpu::Sampler,
    pub linear_repeat: wgpu::Sampler,
    pub comparison: wgpu::Sampler,
}

/// Static GPU resources that are shared across the renderer
pub struct ResourceLibrary {
    pub layout_shared: wgpu::BindGroupLayout,
    pub layout_object: wgpu::BindGroupLayout,
    pub layout_target: wgpu::BindGroupLayout,
    pub layout_light_cull: wgpu::BindGroupLayout,
    pub forward_pipeline_layout: wgpu::PipelineLayout,
    pub forward_shader: wgpu::ShaderModule,
    pub compose_shader: wgpu::ShaderModule,
    pub light_cull_shader: wgpu::ShaderModule,
    pub shadow_shader: wgpu::ShaderModule,
    pub light_cull_pipeline_layout: wgpu::PipelineLayout,
    pub samplers: SamplerSet,
    pub fallback_texture: wgpu::Texture,
    pub fallback_view: wgpu::TextureView,
}

/// Manages uniform pools and current frame bind groups
pub struct BindingSystem {
    pub frame_pool: UniformBufferPool<FrameComponent>,
    pub camera_pool: UniformBufferPool<CameraComponent>,
    pub model_pool: UniformBufferPool<ModelComponent>,
    pub shared_group: Option<wgpu::BindGroup>,
    pub object_group: Option<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightDrawParams {
    camera_index: u32,
    max_lights_per_camera: u32,
}

/// Buffers and state for light preprocessing
pub struct LightCullingSystem {
    pub lights: StorageBufferPool<LightComponent>,
    pub visible_indices: StorageBufferPool<u32>,
    pub visible_counts: StorageBufferPool<u32>,
    pub light_params: UniformBufferPool<LightDrawParams>,
    pub params_buffer: Option<wgpu::Buffer>,
    pub light_count: usize,
    pub camera_count: u32,
    pub max_lights_per_camera: u32,
    pub queue: wgpu::Queue,
}

impl LightCullingSystem {
    pub fn write_draw_params(&mut self, camera_index: u32, max_lights_per_camera: u32) {
        let params = LightDrawParams {
            camera_index,
            max_lights_per_camera,
        };
        self.light_params.write(camera_index, &params);
    }

    pub fn draw_params_offset(&self, camera_index: u32) -> u64 {
        self.light_params.get_offset(camera_index)
    }
}

/// Holds the actual scene data to be rendered
pub struct RenderScene {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,
    pub lights: HashMap<u32, LightRecord>,
}

// -----------------------------------------------------------------------------
// RenderState
// -----------------------------------------------------------------------------

pub struct RenderState {
    pub scene: RenderScene,
    pub bindings: Option<BindingSystem>,
    pub library: Option<ResourceLibrary>,
    pub vertex: Option<VertexAllocatorSystem>,
    pub light_system: Option<LightCullingSystem>,
    pub shadow: Option<ShadowManager>,
    pub cache: RenderCache,
    pub passes: RenderPasses,
}

impl RenderState {
    /// Create a new RenderState with empty systems
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        Self {
            scene: RenderScene {
                cameras: HashMap::new(),
                models: HashMap::new(),
                lights: HashMap::new(),
            },
            bindings: None,
            library: None,
            vertex: None,
            light_system: None,
            shadow: None,
            cache: RenderCache::new(),
            passes: RenderPasses::new(),
        }
    }

    /// Explicitly drop all render state resources
    pub fn drop_all(&mut self) {
        self.scene.cameras.clear();
        self.scene.models.clear();
        self.scene.lights.clear();
        self.bindings = None;
        self.library = None;
        self.vertex = None;
        self.light_system = None;
        self.shadow = None;
        self.cache.clear();
        self.passes = RenderPasses::new();
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        if let Some(vertex) = self.vertex.as_mut() {
            vertex.begin_frame(frame_index);
        }
        if let Some(bindings) = self.bindings.as_mut() {
            bindings.frame_pool.begin_frame(frame_index);
            bindings.camera_pool.begin_frame(frame_index);
            bindings.model_pool.begin_frame(frame_index);
        }
        if let Some(light_system) = self.light_system.as_mut() {
            light_system.lights.begin_frame(frame_index);
            light_system.visible_indices.begin_frame(frame_index);
            light_system.visible_counts.begin_frame(frame_index);
            light_system.light_params.begin_frame(frame_index);
        }
        if let Some(shadow) = self.shadow.as_mut() {
            shadow.begin_frame(frame_index);
        }
        self.cache.gc(frame_index);
    }

    pub fn on_resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        for record in self.scene.cameras.values_mut() {
            let (target_width, target_height) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(width, height))
                .unwrap_or((width, height));

            let size = wgpu::Extent3d {
                width: target_width,
                height: target_height,
                depth_or_array_layers: 1,
            };

            let format = record
                .render_target
                .as_ref()
                .map(|rt| rt.format)
                .unwrap_or(wgpu::TextureFormat::Rgba32Float);

            // Clear old render target before creating new one
            record.clear_render_target();

            let target = RenderTarget::new(device, size, format);
            record.set_render_target(target);

            record
                .data
                .update(None, None, None, None, (target_width, target_height), 10.0);
            record.mark_dirty();
        }

        // Update Forward Pass depth buffer
        let depth_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.passes.forward.depth_target = Some(RenderTarget::new(
            device,
            depth_size,
            wgpu::TextureFormat::Depth24Plus,
        ));
    }

    pub fn prepare_render(&mut self, device: &wgpu::Device, frame_spec: FrameComponent) {
        self.prepare_lights(device);

        let bindings = match self.bindings.as_mut() {
            Some(b) => b,
            None => return,
        };

        let library = match self.library.as_ref() {
            Some(l) => l,
            None => return,
        };

        // 1. Upload all data to pools
        bindings.frame_pool.write(0, &frame_spec);

        for (id, record) in &mut self.scene.cameras {
            if record.is_dirty {
                bindings.camera_pool.write(*id, &record.data);
                record.clear_dirty();
            }
        }

        for (id, record) in &mut self.scene.models {
            if record.is_dirty {
                bindings.model_pool.write(*id, &record.data);
                record.clear_dirty();
            }
        }

        let light_system = match self.light_system.as_ref() {
            Some(sys) => sys,
            None => return,
        };

        let shadow = match self.shadow.as_ref() {
            Some(s) => s,
            None => return,
        };

        // 2. Create Shared Bind Group (Group 0: Frame B0, Camera B1 dynamic, Light params B2 dynamic)
        bindings.shared_group = Some(
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BindGroup Shared (Frame+Camera)"),
                layout: &library.layout_shared,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: bindings.frame_pool.buffer(),
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<FrameComponent>() as u64
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
                                    std::mem::size_of::<CameraComponent>() as u64
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
                                    std::mem::size_of::<LightDrawParams>() as u64
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
                        resource: wgpu::BindingResource::TextureView(shadow.atlas.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: shadow.page_table.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            }),
        );

        // 3. Create Object Bind Group (Group 1: Model B0 dynamic)
        bindings.object_group = Some(
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BindGroup Object (Model)"),
                layout: &library.layout_object,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.model_pool.buffer(),
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<ModelComponent>() as u64)
                                .unwrap(),
                        ),
                    }),
                }],
            }),
        );
    }

    pub(crate) fn init_fallback_resources(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.vertex = Some(VertexAllocatorSystem::new(
            device,
            queue,
            VertexAllocatorConfig::default(),
        ));

        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let storage_alignment = device.limits().min_storage_buffer_offset_alignment as u64;

        // Initialize bindings
        self.bindings = Some(BindingSystem {
            frame_pool: UniformBufferPool::new(device, queue, Some(1), alignment),
            camera_pool: UniformBufferPool::new(device, queue, Some(16), alignment),
            model_pool: UniformBufferPool::new(device, queue, Some(1024), alignment),
            shared_group: None,
            object_group: None,
        });

        self.light_system = Some(LightCullingSystem {
            lights: StorageBufferPool::new(device, queue, Some(32), storage_alignment),
            visible_indices: StorageBufferPool::new(device, queue, Some(128), storage_alignment),
            visible_counts: StorageBufferPool::new(device, queue, Some(8), storage_alignment),
            light_params: UniformBufferPool::new(device, queue, Some(16), alignment),
            params_buffer: Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("LightCull Params"),
                size: std::mem::size_of::<u32>() as u64 * 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            light_count: 0,
            camera_count: 0,
            max_lights_per_camera: 0,
            queue: queue.clone(),
        });

        self.shadow = Some(ShadowManager::new(device, queue, 1024));

        // Initialize fallback texture
        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Texture 1x1"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let white_pixel: [u8; 4] = [255, 255, 255, 255];
        queue.write_texture(
            fallback_texture.as_image_copy(),
            &white_pixel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Initialize samplers
        let samplers = SamplerSet {
            point_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Point Clamp"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            linear_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Linear Clamp"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            point_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Point Repeat"),
                address_mode_u: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            linear_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Linear Repeat"),
                address_mode_u: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            comparison: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Comparison"),
                compare: Some(wgpu::CompareFunction::LessEqual),
                ..Default::default()
            }),
        };

        // Initialize layouts
        let layout_shared = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout Shared"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<FrameComponent>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size:
                            Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<CameraComponent>() as u64
                                )
                                .unwrap(),
                            ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size:
                            Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<LightDrawParams>() as u64
                                )
                                .unwrap(),
                            ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let layout_object = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout Object"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(
                        std::num::NonZeroU64::new(std::mem::size_of::<ModelComponent>() as u64)
                            .unwrap(),
                    ),
                },
                count: None,
            }],
        });

        let layout_target = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout Target"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let layout_light_cull = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout Light Cull"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64 * 4)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
            ],
        });

        // Initialize forward pass resources
        let forward_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Forward Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "passes/forward/forward.wgsl"
            ))),
        });

        let compose_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compose Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "passes/compose/compose.wgsl"
            ))),
        });

        let light_cull_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Light Cull Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "passes/light_cull/light_cull.wgsl"
            ))),
        });

        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "passes/shadow/shadow.wgsl"
            ))),
        });

        let forward_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward Pipeline Layout"),
                bind_group_layouts: &[&layout_shared, &layout_object],
                push_constant_ranges: &[],
            });

        let light_cull_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Cull Pipeline Layout"),
                bind_group_layouts: &[&layout_light_cull],
                push_constant_ranges: &[],
            });

        self.library = Some(ResourceLibrary {
            layout_shared,
            layout_object,
            layout_target,
            layout_light_cull,
            forward_shader,
            forward_pipeline_layout,
            compose_shader,
            light_cull_shader,
            shadow_shader,
            light_cull_pipeline_layout,
            samplers,
            fallback_texture,
            fallback_view,
        });
    }

    fn prepare_lights(&mut self, device: &wgpu::Device) {
        let light_system = match self.light_system.as_mut() {
            Some(sys) => sys,
            None => return,
        };

        let mut lights = Vec::with_capacity(self.scene.lights.len());
        for record in self.scene.lights.values() {
            lights.push(record.data);
        }

        light_system.light_count = lights.len();
        if lights.is_empty() {
            return;
        }

        light_system.lights.write_slice(0, &lights);

        let camera_count = self.scene.cameras.len() as u32;
        let total_indices = (lights.len() as u32) * camera_count;
        if total_indices > 0 {
            let zeros = vec![0u32; total_indices as usize];
            light_system.visible_indices.write_slice(0, &zeros);
        }

        if camera_count > 0 {
            let zeros = vec![0u32; camera_count as usize];
            light_system.visible_counts.write_slice(0, &zeros);
        }

        if light_system.params_buffer.is_none() {
            light_system.params_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("LightCull Params"),
                size: std::mem::size_of::<u32>() as u64 * 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }
}
