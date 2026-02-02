#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod fallbacks;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod library;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod systems;

#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use crate::core::render::state::{RenderState, ResourceLibrary};

#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
impl RenderState {
    pub(crate) fn init(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _surface_format: wgpu::TextureFormat,
    ) {
        // 1. Initialize core systems
        self.init_core_systems(device, queue);

        // 2. Initialize samplers
        let samplers = self.init_samplers(device);

        // 3. Initialize layouts
        let layouts = self.init_layouts(device);

        // 4. Initialize fallback textures
        let fallbacks = self.init_fallback_textures(device, queue);

        // 5. Initialize pipeline layouts
        let gizmo_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gizmo Pipeline Layout"),
                bind_group_layouts: &[&layouts.shared],
                immediate_size: 0,
            });

        let forward_standard_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward Standard Pipeline Layout"),
                bind_group_layouts: &[&layouts.shared, &layouts.object_standard],
                immediate_size: 0,
            });

        let forward_pbr_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Forward PBR Pipeline Layout"),
                bind_group_layouts: &[&layouts.shared, &layouts.object_pbr],
                immediate_size: 0,
            });

        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[&layouts.shared, &layouts.object],
                immediate_size: 0,
            });

        let outline_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outline Pipeline Layout"),
                bind_group_layouts: &[&layouts.shared, &layouts.object],
                immediate_size: 0,
            });

        let light_cull_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LightCull Pipeline Layout"),
                bind_group_layouts: &[&layouts.light_cull],
                immediate_size: 0,
            });

        let ssao_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[&layouts.ssao],
            immediate_size: 0,
        });

        let ssao_blur_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SSAO Blur Pipeline Layout"),
                bind_group_layouts: &[&layouts.ssao_blur],
                immediate_size: 0,
            });

        let ssao_msaa_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SSAO MSAA Pipeline Layout"),
                bind_group_layouts: &[&layouts.ssao_msaa],
                immediate_size: 0,
            });

        let ssao_blur_msaa_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SSAO Blur MSAA Pipeline Layout"),
                bind_group_layouts: &[&layouts.ssao_blur_msaa],
                immediate_size: 0,
            });

        let bloom_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Pipeline Layout"),
                bind_group_layouts: &[&layouts.bloom],
                immediate_size: 0,
            });

        // 6. Initialize shaders
        let forward_standard_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_standard.wgsl"
        ));
        let forward_pbr_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_pbr.wgsl"
        ));
        let post_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/post/post.wgsl"));
        let compose_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/compose/compose.wgsl"));
        let light_cull_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/light_cull/light_cull.wgsl"
        ));
        let shadow_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/shadow/shadow.wgsl"));
        let outline_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/outline/outline.wgsl"));
        let ssao_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao.wgsl"));
        let ssao_blur_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_blur.wgsl"));
        let ssao_msaa_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_msaa.wgsl"));
        let ssao_blur_msaa_shader = device
            .create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_blur_msaa.wgsl"));
        let bloom_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/bloom/bloom.wgsl"));
        let gizmo_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../gizmos/gizmo.wgsl"));

        let post_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PostProcess Uniform Buffer"),
            size: 96,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ssao_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Uniform Buffer"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ssao_blur_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Blur Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bloom_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bloom Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 7. Initialize library
        self.library = Some(ResourceLibrary {
            layout_shared: layouts.shared,
            layout_object: layouts.object,
            layout_object_standard: layouts.object_standard,
            layout_object_pbr: layouts.object_pbr,
            layout_target: layouts.target,
            layout_light_cull: layouts.light_cull,
            layout_ssao: layouts.ssao,
            layout_ssao_blur: layouts.ssao_blur,
            layout_ssao_msaa: layouts.ssao_msaa,
            layout_ssao_blur_msaa: layouts.ssao_blur_msaa,
            layout_bloom: layouts.bloom,
            forward_standard_pipeline_layout,
            forward_pbr_pipeline_layout,
            shadow_pipeline_layout,
            outline_pipeline_layout,
            ssao_pipeline_layout,
            ssao_blur_pipeline_layout,
            ssao_msaa_pipeline_layout,
            ssao_blur_msaa_pipeline_layout,
            bloom_pipeline_layout,
            forward_standard_shader,
            forward_pbr_shader,
            post_shader,
            compose_shader,
            outline_shader,
            ssao_shader,
            ssao_blur_shader,
            ssao_msaa_shader,
            ssao_blur_msaa_shader,
            bloom_shader,
            light_cull_shader,
            shadow_shader,
            gizmo_shader,
            light_cull_pipeline_layout,
            gizmo_pipeline_layout,
            samplers,
            _fallback_texture: fallbacks.texture,
            fallback_view: fallbacks.view,
            _fallback_forward_atlas_texture: fallbacks.atlas_texture,
            fallback_forward_atlas_view: fallbacks.atlas_view,
            _fallback_shadow_texture: fallbacks.shadow_texture,
            fallback_shadow_view: fallbacks.shadow_view,
        });

        self.post_uniform_buffer = Some(post_uniform_buffer);
        self.ssao_uniform_buffer = Some(ssao_uniform_buffer);
        self.ssao_blur_uniform_buffer = Some(ssao_blur_uniform_buffer);
        self.bloom_uniform_buffer = Some(bloom_uniform_buffer);
    }
}
