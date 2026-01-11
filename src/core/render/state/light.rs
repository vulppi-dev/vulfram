use crate::core::resources::{LightComponent, StorageBufferPool, UniformBufferPool};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightDrawParams {
    pub camera_index: u32,
    pub max_lights_per_camera: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrustumPlane {
    pub data: glam::Vec4,
}

/// Buffers and state for light preprocessing
pub struct LightCullingSystem {
    pub lights: StorageBufferPool<LightComponent>,
    pub visible_indices: StorageBufferPool<u32>,
    pub visible_counts: StorageBufferPool<u32>,
    pub camera_frustums: StorageBufferPool<FrustumPlane>,
    pub light_params: UniformBufferPool<LightDrawParams>,
    pub params_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub light_count: usize,
    pub camera_count: u32,
    pub max_lights_per_camera: u32,
    pub queue: wgpu::Queue,
}

impl LightCullingSystem {
    pub fn write_draw_params(&mut self, camera_index: u32, max_lights_per_camera: u32) {
        let params = LightDrawParams {
            camera_index,
            max_lights_per_camera,
        };
        self.light_params.write(camera_index, &params);
    }

    pub fn draw_params_offset(&self, camera_index: u32) -> u64 {
        self.light_params.get_offset(camera_index)
    }
}
