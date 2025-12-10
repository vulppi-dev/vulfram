use super::binding::BindingManager;
use super::components::Components;
use super::pipeline::PipelineCache;
use super::resources::Resources;

pub struct RenderState {
    pub components: Components,
    pub resources: Resources,
    pub clear_color: wgpu::Color,

    /// Surface texture format (used for pipeline creation)
    pub surface_format: wgpu::TextureFormat,

    /// Binding manager for component-shader-resource combinations
    pub binding_manager: BindingManager,

    /// Pipeline cache for shader-material combinations
    pub pipeline_cache: PipelineCache,

    /// Blit pipeline for compositing camera render targets to surface
    pub blit_pipeline: Option<wgpu::RenderPipeline>,

    /// Blit sampler for texture sampling
    pub blit_sampler: Option<wgpu::Sampler>,

    /// Blit bind group layout
    pub blit_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl RenderState {
    /// Create a new RenderState with the specified surface format
    pub fn new(surface_format: wgpu::TextureFormat) -> Self {
        Self {
            components: Components::new(),
            resources: Resources::new(),
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
            surface_format,
            binding_manager: BindingManager::new(),
            pipeline_cache: PipelineCache::new(),
            blit_pipeline: None,
            blit_sampler: None,
            blit_bind_group_layout: None,
        }
    }

    /// Initialize blit resources (pipeline, sampler, bind group layout)
    pub fn init_blit_resources(&mut self, device: &wgpu::Device) {
        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
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

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blit.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        self.blit_pipeline = Some(pipeline);
        self.blit_sampler = Some(sampler);
        self.blit_bind_group_layout = Some(bind_group_layout);
    }

    /// Explicitly drop all render state resources
    /// This ensures proper cleanup of GPU resources
    pub fn drop_all(&mut self) {
        // Clear caches first
        self.binding_manager.clear();
        self.pipeline_cache.clear();

        // Drop components (includes render targets)
        self.components.drop_all();

        // Drop resources (includes shaders with their buffers)
        self.resources.drop_all();
    }
}
