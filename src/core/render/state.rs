use std::collections::HashMap;

use crate::core::render::cache::RenderCache;
use crate::core::render::passes::RenderPasses;
use crate::core::resources::{
    CameraComponent, CameraRecord, FrameComponent, ModelComponent, ModelRecord, RenderTarget,
    UniformBufferPool, VertexAllocatorConfig, VertexAllocatorSystem,
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
    pub forward_pipeline_layout: wgpu::PipelineLayout,
    pub forward_shader: wgpu::ShaderModule,
    pub compose_shader: wgpu::ShaderModule,
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

/// Holds the actual scene data to be rendered
pub struct RenderScene {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,
}

// -----------------------------------------------------------------------------
// RenderState
// -----------------------------------------------------------------------------

pub struct RenderState {
    pub scene: RenderScene,
    pub bindings: Option<BindingSystem>,
    pub library: Option<ResourceLibrary>,
    pub vertex: Option<VertexAllocatorSystem>,
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
            },
            bindings: None,
            library: None,
            vertex: None,
            cache: RenderCache::new(),
            passes: RenderPasses::new(),
        }
    }

    /// Explicitly drop all render state resources
    pub fn drop_all(&mut self) {
        self.scene.cameras.clear();
        self.scene.models.clear();
        self.bindings = None;
        self.library = None;
        self.vertex = None;
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

        // 2. Create Shared Bind Group (Group 0: Frame B0, Camera B1 dynamic)
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

        // Initialize bindings
        self.bindings = Some(BindingSystem {
            frame_pool: UniformBufferPool::new(device, queue, Some(1), alignment),
            camera_pool: UniformBufferPool::new(device, queue, Some(16), alignment),
            model_pool: UniformBufferPool::new(device, queue, Some(1024), alignment),
            shared_group: None,
            object_group: None,
        });

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

        let forward_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward Pipeline Layout"),
                bind_group_layouts: &[&layout_shared, &layout_object],
                push_constant_ranges: &[],
            });

        self.library = Some(ResourceLibrary {
            layout_shared,
            layout_object,
            layout_target,
            forward_shader,
            forward_pipeline_layout,
            compose_shader,
            samplers,
            fallback_texture,
            fallback_view,
        });
    }
}
