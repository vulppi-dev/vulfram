use super::super::{BindingSystem, LightCullingSystem, RenderState, SamplerSet};
use crate::core::resources::shadow::ShadowManager;
use crate::core::resources::{
    StorageBufferPool, UniformBufferPool, VertexAllocatorConfig, VertexAllocatorSystem,
};
use std::collections::HashMap;

impl RenderState {
    pub(crate) fn init_core_systems(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
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
            camera_pool: UniformBufferPool::new(device, queue, Some(128), alignment),
            model_pool: UniformBufferPool::new(device, queue, Some(2048), alignment),
            instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            outline_instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            shadow_instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            material_standard_pool: UniformBufferPool::new(device, queue, Some(256), alignment),
            material_standard_inputs: StorageBufferPool::new(device, queue, Some(256), 0),
            material_pbr_pool: UniformBufferPool::new(device, queue, Some(256), alignment),
            material_pbr_inputs: StorageBufferPool::new(device, queue, Some(256), 0),
            bones_pool: StorageBufferPool::new(device, queue, Some(256), 0),
            shared_group: None,
            model_bind_group: None,
            outline_model_bind_group: None,
            shadow_model_bind_group: None,
            pool_versions: HashMap::new(),
            last_with_shadows: false,
        });

        self.light_system = Some(LightCullingSystem {
            lights: StorageBufferPool::new(device, queue, Some(32), storage_alignment),
            visible_indices: StorageBufferPool::new(device, queue, Some(128), storage_alignment),
            visible_counts: StorageBufferPool::new(device, queue, Some(8), storage_alignment),
            camera_frustums: StorageBufferPool::new(device, queue, Some(96), storage_alignment),
            light_params: UniformBufferPool::new(device, queue, Some(16), alignment),
            params_buffer: Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("LightCull Params"),
                size: std::mem::size_of::<u32>() as u64 * 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            bind_group: None,
            light_count: 0,
            camera_count: 0,
            max_lights_per_camera: 0,
            queue: queue.clone(),
        });

        self.shadow = Some(ShadowManager::new(device, queue, 2048));
    }

    pub(crate) fn init_samplers(&mut self, device: &wgpu::Device) -> SamplerSet {
        SamplerSet {
            point_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Point Clamp"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            linear_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Linear Clamp"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            point_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Point Repeat"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            linear_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Linear Repeat"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            comparison: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Sampler Comparison"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                compare: Some(wgpu::CompareFunction::GreaterEqual), // Reverse Z
                ..Default::default()
            }),
        }
    }
}
