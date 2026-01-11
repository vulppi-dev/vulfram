use crate::core::resources::{
    CameraComponent, FrameComponent, MaterialPbrParams, MaterialStandardParams, ModelComponent,
    StorageBufferPool, UniformBufferPool,
};
use std::collections::HashMap;

/// Manages uniform pools and current frame bind groups
pub struct BindingSystem {
    pub frame_pool: UniformBufferPool<FrameComponent>,
    pub camera_pool: UniformBufferPool<CameraComponent>,
    pub model_pool: UniformBufferPool<ModelComponent>,
    pub instance_pool: StorageBufferPool<ModelComponent>,
    pub shadow_instance_pool: StorageBufferPool<ModelComponent>,
    pub material_standard_pool: UniformBufferPool<MaterialStandardParams>,
    pub material_standard_inputs: StorageBufferPool<glam::Vec4>,
    pub material_pbr_pool: UniformBufferPool<MaterialPbrParams>,
    pub material_pbr_inputs: StorageBufferPool<glam::Vec4>,
    pub shared_group: Option<wgpu::BindGroup>,
    pub model_bind_group: Option<wgpu::BindGroup>,
    pub shadow_model_bind_group: Option<wgpu::BindGroup>,

    // Version tracking for bind group invalidation
    pub pool_versions: HashMap<&'static str, u64>,
    pub last_with_shadows: bool,
}
