use glam::{Mat4, Vec2};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use wgpu;

use super::resources::{GeometryId, MaterialId};

// MARK: - Logical IDs

pub type ComponentId = u32;

// MARK: - Viewport

/// Viewport size/position mode
///
/// Allows mixing relative and absolute coordinates for maximum flexibility.
/// Example use cases:
/// - Relative position + Absolute size: UI button at relative screen position with fixed pixel size
/// - Absolute position + Relative size: Content area with pixel margin and scalable size
/// - Relative + Relative: Fullscreen or percentage-based layouts
/// - Absolute + Absolute: Fixed position and size (traditional pixel-based UI)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize_repr, Serialize_repr)]
pub enum ViewportMode {
    /// Relative (0.0 to 1.0 normalized coordinates)
    Relative = 0,
    /// Absolute (pixel coordinates)
    Absolute,
}

/// Viewport configuration for camera rendering
///
/// The viewport system supports independent positioning and sizing modes,
/// allowing you to mix relative and absolute coordinates. This provides
/// maximum flexibility for different use cases:
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Viewport {
    /// Position mode (relative or absolute)
    pub position_mode: ViewportMode,
    /// Size mode (relative or absolute)
    pub size_mode: ViewportMode,
    /// X position (normalized 0-1 if relative, pixels if absolute)
    pub x: f32,
    /// Y position (normalized 0-1 if relative, pixels if absolute)
    pub y: f32,
    /// Width (normalized 0-1 if relative, pixels if absolute)
    pub width: f32,
    /// Height (normalized 0-1 if relative, pixels if absolute)
    pub height: f32,
    /// Anchor point for positioning (0,0 = top-left, 1,1 = bottom-right)
    pub anchor: Vec2,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            position_mode: ViewportMode::Relative,
            size_mode: ViewportMode::Relative,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            anchor: Vec2::ZERO, // Top-left by default
        }
    }
}

// MARK: - Camera

/// CameraInstance represents a camera component attached to an entity
pub struct CameraInstance {
    pub viewport: Viewport,
    /// Projection matrix (perspective or orthographic)
    pub proj_mat: Mat4,
    /// View matrix (camera transform inverse)
    pub view_mat: Mat4,
    pub render_target: wgpu::Texture,
    pub render_target_view: wgpu::TextureView,
    pub layer_mask: u32,
    /// Dirty flag indicating component needs GPU buffer update
    pub is_dirty: bool,
}

// MARK: - Mesh/Model

/// MeshInstance represents a mesh/model component attached to an entity
pub struct MeshInstance {
    pub geometry: GeometryId,
    pub material: MaterialId,
    /// Model transformation matrix (position, rotation, scale)
    pub model_mat: Mat4,
    pub layer_mask: u32,
    /// Dirty flag indicating component needs GPU buffer update
    pub is_dirty: bool,
}

// MARK: - Components Manager

/// Components holds all component instances indexed by ComponentId
pub struct Components {
    pub cameras: HashMap<ComponentId, CameraInstance>,
    pub models: HashMap<ComponentId, MeshInstance>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            cameras: HashMap::new(),
            models: HashMap::new(),
        }
    }

    /// Explicitly drop all components and their GPU resources
    /// This ensures proper cleanup of render targets and other GPU resources
    pub fn drop_all(&mut self) {
        // Drop camera render targets explicitly before clearing
        // This prevents GPU memory leaks from render target textures
        for (_, camera) in self.cameras.drain() {
            drop(camera.render_target_view);
            drop(camera.render_target);
        }
        self.models.clear();
    }
}

impl Default for Components {
    fn default() -> Self {
        Self::new()
    }
}
