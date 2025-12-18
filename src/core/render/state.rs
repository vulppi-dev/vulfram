use std::collections::HashMap;

use crate::core::resources::{CameraComponent, ComponentContainer};

pub struct RenderState {
    pub cameras: HashMap<u32, ComponentContainer<CameraComponent>>,
}

impl RenderState {
    /// Create a new RenderState with the specified surface format
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        Self {
            cameras: HashMap::new(),
        }
    }

    /// Explicitly drop all render state resources
    /// This ensures proper cleanup of GPU resources
    pub fn drop_all(&mut self) {
        self.cameras.clear();
    }

    pub(crate) fn init_fallback_resources(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> () {
        // TODO: Initialize fallback resources (default shader, texture, geometry)
    }
}
