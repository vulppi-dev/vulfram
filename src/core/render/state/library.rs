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
    pub layout_object_standard: wgpu::BindGroupLayout,
    pub layout_object_pbr: wgpu::BindGroupLayout,
    pub layout_target: wgpu::BindGroupLayout,
    pub layout_light_cull: wgpu::BindGroupLayout,
    pub forward_standard_pipeline_layout: wgpu::PipelineLayout,
    pub forward_pbr_pipeline_layout: wgpu::PipelineLayout,
    pub shadow_pipeline_layout: wgpu::PipelineLayout,
    pub forward_standard_shader: wgpu::ShaderModule,
    pub forward_pbr_shader: wgpu::ShaderModule,
    pub compose_shader: wgpu::ShaderModule,
    pub light_cull_shader: wgpu::ShaderModule,
    pub shadow_shader: wgpu::ShaderModule,
    pub gizmo_shader: wgpu::ShaderModule,
    pub light_cull_pipeline_layout: wgpu::PipelineLayout,
    pub gizmo_pipeline_layout: wgpu::PipelineLayout,
    pub samplers: SamplerSet,
    pub _fallback_texture: wgpu::Texture,
    pub fallback_view: wgpu::TextureView,
    pub _fallback_forward_atlas_texture: wgpu::Texture,
    pub fallback_forward_atlas_view: wgpu::TextureView,
    pub _fallback_shadow_texture: wgpu::Texture,
    pub fallback_shadow_view: wgpu::TextureView,
}
