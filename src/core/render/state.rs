use std::collections::HashMap;

use crate::core::resources::{
    CameraComponent, CameraRecord, ModelComponent, ModelRecord, UniformBufferPool,
    VertexAllocatorConfig, VertexAllocatorSystem,
};

pub struct RenderState {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,

    // Buffers
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
}

impl RenderState {
    /// Create a new RenderState with the specified surface format
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        Self {
            cameras: HashMap::new(),
            models: HashMap::new(),
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
    }
}
