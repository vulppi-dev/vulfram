use std::collections::HashMap;

use crate::core::resources::{
    CameraComponent, CameraRecord, FrameSpec, ModelComponent, ModelRecord, RenderTarget,
    UniformBufferPool, VertexAllocatorConfig, VertexAllocatorSystem,
};
use crate::core::render::cache::RenderCache;

pub struct RenderState {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,

    // Buffers
    pub frame_buffer: Option<UniformBufferPool<FrameSpec>>,
    pub camera_buffer: Option<UniformBufferPool<CameraComponent>>,
    pub model_buffer: Option<UniformBufferPool<ModelComponent>>,

    // Vertex System
    pub vertex_allocation: Option<VertexAllocatorSystem>,

    // Fallback resources
    pub fallback_texture: Option<wgpu::Texture>,
    pub fallback_texture_view: Option<wgpu::TextureView>,

    // Reusable samplers
    pub sampler_point_clamp: Option<wgpu::Sampler>,
    pub sampler_linear_clamp: Option<wgpu::Sampler>,
    pub sampler_point_repeat: Option<wgpu::Sampler>,
    pub sampler_linear_repeat: Option<wgpu::Sampler>,
    pub sampler_comparison: Option<wgpu::Sampler>,

    // Pipeline cache
    pub render_cache: RenderCache,

    // Bind Group Layouts
    pub layout_shared: Option<wgpu::BindGroupLayout>,
    pub layout_object: Option<wgpu::BindGroupLayout>,
    pub layout_target: Option<wgpu::BindGroupLayout>,

    // Bind Groups (Updated per frame)
    pub bind_group_shared: Option<wgpu::BindGroup>,
    pub bind_group_object: Option<wgpu::BindGroup>,
}

impl RenderState {
    /// Create a new RenderState with the specified surface format
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        Self {
            cameras: HashMap::new(),
            models: HashMap::new(),
            frame_buffer: None,
            camera_buffer: None,
            model_buffer: None,
            vertex_allocation: None,
            fallback_texture: None,
            fallback_texture_view: None,
            sampler_point_clamp: None,
            sampler_linear_clamp: None,
            sampler_point_repeat: None,
            sampler_linear_repeat: None,
            sampler_comparison: None,
            render_cache: RenderCache::new(),
            layout_shared: None,
            layout_object: None,
            layout_target: None,
            bind_group_shared: None,
            bind_group_object: None,
        }
    }

    /// Explicitly drop all render state resources
    /// This ensures proper cleanup of GPU resources
    pub fn drop_all(&mut self) {
        self.cameras.clear();
        self.models.clear();
        self.fallback_texture = None;
        self.fallback_texture_view = None;
        self.sampler_point_clamp = None;
        self.sampler_linear_clamp = None;
        self.sampler_point_repeat = None;
        self.sampler_linear_repeat = None;
        self.sampler_comparison = None;
        self.render_cache.clear();
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        if let Some(vertex_allocator) = self.vertex_allocation.as_mut() {
            vertex_allocator.begin_frame(frame_index);
        }
        if let Some(frame_buffer) = self.frame_buffer.as_mut() {
            frame_buffer.begin_frame(frame_index);
        }
        if let Some(camera_buffer) = self.camera_buffer.as_mut() {
            camera_buffer.begin_frame(frame_index);
        }
        if let Some(model_buffer) = self.model_buffer.as_mut() {
            model_buffer.begin_frame(frame_index);
        }
        self.render_cache.gc(frame_index);
    }

    pub fn on_resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        for record in self.cameras.values_mut() {
            // Resolve new size based on window dimensions
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

            // Get existing format or fallback to Rgba8UnormSrgb
            let format = record
                .render_target
                .as_ref()
                .map(|rt| rt.format)
                .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

            // Re-create render target with new size
            let target = RenderTarget::new(device, size, format);
            record.set_render_target(target);

            // Update camera projection to match new aspect ratio/size
            let viewport = glam::Vec4::new(target_width as f32, target_height as f32, 0.0, 0.0);
            record.data.update(None, None, None, None, viewport);

            record.mark_dirty();
        }
    }

    pub fn prepare_render(
        &mut self,
        device: &wgpu::Device,
        frame_spec: FrameSpec,
    ) {
        // 1. Upload all data to pools
        if let Some(pool) = self.frame_buffer.as_mut() {
            pool.write(0, &frame_spec);
        }

        if let Some(pool) = self.camera_buffer.as_mut() {
            for (id, record) in &self.cameras {
                pool.write(*id, &record.data);
            }
        }

        if let Some(pool) = self.model_buffer.as_mut() {
            for (id, record) in &self.models {
                pool.write(*id, &record.data);
            }
        }

        // 2. Create Shared Bind Group (Group 0: Frame B0, Camera B1 dynamic)
        if let (Some(frame_pool), Some(camera_pool), Some(layout)) = (
            self.frame_buffer.as_ref(),
            self.camera_buffer.as_ref(),
            self.layout_shared.as_ref(),
        ) {
            self.bind_group_shared = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BindGroup Shared (Frame+Camera)"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: frame_pool.buffer(),
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(std::mem::size_of::<FrameSpec>() as u64)
                                    .unwrap(),
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: camera_pool.buffer(),
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<CameraComponent>() as u64,
                                )
                                .unwrap(),
                            ),
                        }),
                    },
                ],
            }));
        }

        // 3. Create Object Bind Group (Group 1: Model B0 dynamic)
        if let (Some(pool), Some(layout)) =
            (self.model_buffer.as_ref(), self.layout_object.as_ref())
        {
            self.bind_group_object = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BindGroup Object (Model)"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: pool.buffer(),
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<ModelComponent>() as u64)
                                .unwrap(),
                        ),
                    }),
                }],
            }));
        }
    }

    pub(crate) fn init_fallback_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> () {
        self.vertex_allocation = Some(VertexAllocatorSystem::new(
            device,
            queue,
            VertexAllocatorConfig::default(),
        ));

        // Initialize uniform pools
        self.frame_buffer = Some(UniformBufferPool::new(device, queue, Some(1)));
        self.camera_buffer = Some(UniformBufferPool::new(device, queue, Some(16)));
        self.model_buffer = Some(UniformBufferPool::new(device, queue, Some(1024)));

        // Create 1x1 white fallback texture
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

        // Write white pixel to fallback texture
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

        let fallback_texture_view =
            fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create point sampling with clamp addressing
        let sampler_point_clamp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Point Clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create linear sampling with clamp addressing
        let sampler_linear_clamp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear Clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create point sampling with repeat addressing
        let sampler_point_repeat = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Point Repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create linear sampling with repeat addressing
        let sampler_linear_repeat = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear Repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create comparison sampler for shadow mapping
        let sampler_comparison = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Comparison (Shadow)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        self.fallback_texture = Some(fallback_texture);
        self.fallback_texture_view = Some(fallback_texture_view);
        self.sampler_point_clamp = Some(sampler_point_clamp);
        self.sampler_linear_clamp = Some(sampler_linear_clamp);
        self.sampler_point_repeat = Some(sampler_point_repeat);
        self.sampler_linear_repeat = Some(sampler_linear_repeat);
        self.sampler_comparison = Some(sampler_comparison);

        // Create standard bind group layouts
        // Group 0: Shared (Frame + Camera)
        let layout_shared = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout Shared"),
            entries: &[
                // Binding 0: Frame Spec (Static)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<FrameSpec>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
                // Binding 1: Camera (Dynamic Offset)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<CameraComponent>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
            ],
        });

        // Group 1: Object (Model)
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        self.layout_shared = Some(layout_shared);
        self.layout_object = Some(layout_object);
        self.layout_target = Some(layout_target);
    }
}
