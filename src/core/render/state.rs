use std::collections::HashMap;

use crate::core::resources::{
    ArenaAllocator, CameraComponent, ComponentContainer, UniformBufferPool,
};

pub struct RenderState {
    pub cameras: HashMap<u32, ComponentContainer<CameraComponent>>,

    // Buffers
    pub camera_buffer: Option<UniformBufferPool<CameraComponent>>,
    // pub dummy_buffer: Option<wgpu::Buffer>,

    // Arenas
    pub positions_arena: Option<ArenaAllocator>,
    pub normals_arena: Option<ArenaAllocator>,
    pub colors_arena: Option<ArenaAllocator>,
    pub tangents_arena: Option<ArenaAllocator>,
    pub uv_arena: Option<ArenaAllocator>,
    pub joints_arena: Option<ArenaAllocator>,
    pub weights_arena: Option<ArenaAllocator>,
    pub indices_arena: Option<ArenaAllocator>,

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
            camera_buffer: None,
            // dummy_buffer: None,
            positions_arena: None,
            normals_arena: None,
            colors_arena: None,
            tangents_arena: None,
            uv_arena: None,
            joints_arena: None,
            weights_arena: None,
            indices_arena: None,
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
        // // Create dummy buffer for bindings that require a buffer but have no data
        // let dummy_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Dummy Buffer"),
        //     size: 0,
        //     usage: wgpu::BufferUsages::VERTEX
        //         | wgpu::BufferUsages::INDEX
        //         | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });
        // self.dummy_buffer = Some(dummy_buffer);

        // Arenas
        let arena_initial_capacity = 1024 * 1024; // 1 MB
        self.positions_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Positions Arena"),
        ));
        self.normals_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Normals Arena"),
        ));
        self.colors_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Colors Arena"),
        ));
        self.tangents_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Tangents Arena"),
        ));
        self.uv_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("UV Arena"),
        ));
        self.joints_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Joints Arena"),
        ));
        self.weights_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::VERTEX,
            Some("Weights Arena"),
        ));
        self.indices_arena = Some(ArenaAllocator::new(
            device,
            queue,
            arena_initial_capacity,
            wgpu::BufferUsages::INDEX,
            Some("Indices Arena"),
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
