mod fallbacks;
mod library;
mod systems;

use crate::core::render::state::{RenderState, ResourceLibrary};

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

        let light_cull_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LightCull Pipeline Layout"),
                bind_group_layouts: &[&layouts.light_cull],
                immediate_size: 0,
            });

        // 6. Initialize shaders
        let forward_standard_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_standard.wgsl"
        ));
        let forward_pbr_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_pbr.wgsl"
        ));
        let compose_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/compose/compose.wgsl"));
        let light_cull_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/light_cull/light_cull.wgsl"
        ));
        let shadow_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/shadow/shadow.wgsl"));
        let gizmo_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../gizmos/gizmo.wgsl"));

        // 7. Initialize library
        self.library = Some(ResourceLibrary {
            layout_shared: layouts.shared,
            layout_object: layouts.object,
            layout_object_standard: layouts.object_standard,
            layout_object_pbr: layouts.object_pbr,
            layout_target: layouts.target,
            layout_light_cull: layouts.light_cull,
            forward_standard_pipeline_layout,
            forward_pbr_pipeline_layout,
            shadow_pipeline_layout,
            forward_standard_shader,
            forward_pbr_shader,
            compose_shader,
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
    }
}
