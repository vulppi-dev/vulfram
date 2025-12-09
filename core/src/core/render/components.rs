use glam::Mat4;
use std::collections::HashMap;
use wgpu;

use super::resources::{GeometryId, MaterialId};

// MARK: - Logical IDs

pub type EntityId = u32;

// MARK: - Viewport

/// Viewport mode for camera rendering
#[derive(Debug, Clone)]
pub enum Viewport {
    /// Relative viewport (0.0 to 1.0 range)
    Relative {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Absolute viewport (pixel coordinates)
    Absolute {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

/// ViewportDesc for command deserialization
#[derive(Debug, Clone)]
pub struct ViewportDesc {
    pub mode: ViewportMode,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub enum ViewportMode {
    Relative,
    Absolute,
}

// MARK: - Camera

/// CameraDesc for camera creation
#[derive(Debug, Clone)]
pub struct CameraDesc {
    pub proj_mat: Mat4,
    pub view_mat: Mat4,
}

/// CameraInstance represents a camera component attached to an entity
pub struct CameraInstance {
    pub camera_uniform_offset: u32,
    pub viewport: Viewport,
    pub render_target: wgpu::Texture,
    pub render_target_view: wgpu::TextureView,
    pub layer_mask: u32,
}

// MARK: - Mesh/Model

/// MeshInstance represents a mesh/model component attached to an entity
pub struct MeshInstance {
    pub geometry: GeometryId,
    pub material: MaterialId,
    pub model_uniform_offset: u32,
    pub layer_mask: u32,
}

// MARK: - Components Manager

/// Components holds all component instances indexed by EntityId
pub struct Components {
    pub cameras: HashMap<EntityId, CameraInstance>,
    pub models: HashMap<EntityId, MeshInstance>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            cameras: HashMap::new(),
            models: HashMap::new(),
        }
    }

    pub fn drop_all(&mut self) {
        self.cameras.clear();
        self.models.clear();
    }
}

impl Default for Components {
    fn default() -> Self {
        Self::new()
    }
}
