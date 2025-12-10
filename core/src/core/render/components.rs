use glam::{Mat4, Vec2};
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
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ViewportMode {
    /// Relative (0.0 to 1.0 normalized coordinates)
    Relative,
    /// Absolute (pixel coordinates)
    Absolute,
}

/// Viewport configuration for camera rendering
///
/// The viewport system supports independent positioning and sizing modes,
/// allowing you to mix relative and absolute coordinates. This provides
/// maximum flexibility for different use cases:
///
/// # Examples
///
/// ```rust,ignore
/// // Fullscreen viewport (default)
/// let viewport = Viewport::fullscreen();
///
/// // Fixed 800x600 centered window
/// let viewport = Viewport::centered_absolute(800, 600);
///
/// // UI button: 50% from left, 10% from top, fixed 200x50 pixels
/// let viewport = Viewport::relative_pos_absolute_size(0.5, 0.1, 200, 50);
///
/// // Content with 10px margin, scales with window
/// let viewport = Viewport::absolute_pos_relative_size(10, 10, 0.95, 0.95);
///
/// // Custom anchor (0,0 = top-left, 0.5,0.5 = center, 1,1 = bottom-right)
/// let viewport = Viewport {
///     position_mode: ViewportMode::Relative,
///     size_mode: ViewportMode::Relative,
///     x: 1.0,  // Right edge
///     y: 1.0,  // Bottom edge
///     width: 0.25,  // 25% width
///     height: 0.25, // 25% height
///     anchor: Vec2::new(1.0, 1.0), // Anchor at bottom-right
/// };
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

impl Viewport {
    /// Calculate actual pixel rectangle based on window size
    /// Returns UVec2 (position) and UVec2 (size) in pixels
    pub fn calculate_rect(
        &self,
        window_width: u32,
        window_height: u32,
    ) -> (glam::UVec2, glam::UVec2) {
        // Calculate position
        let (px, py) = match self.position_mode {
            ViewportMode::Relative => (
                (self.x * window_width as f32) as u32,
                (self.y * window_height as f32) as u32,
            ),
            ViewportMode::Absolute => (self.x as u32, self.y as u32),
        };

        // Calculate size
        let (w, h) = match self.size_mode {
            ViewportMode::Relative => (
                (self.width * window_width as f32) as u32,
                (self.height * window_height as f32) as u32,
            ),
            ViewportMode::Absolute => (self.width as u32, self.height as u32),
        };

        // Apply anchor offset
        let anchor_offset_x = (w as f32 * self.anchor.x) as u32;
        let anchor_offset_y = (h as f32 * self.anchor.y) as u32;

        let final_x = px.saturating_sub(anchor_offset_x);
        let final_y = py.saturating_sub(anchor_offset_y);

        (glam::UVec2::new(final_x, final_y), glam::UVec2::new(w, h))
    }

    /// Create a fullscreen viewport (covers entire window)
    pub fn fullscreen() -> Self {
        Self::default()
    }

    /// Create a viewport with relative position and absolute size
    /// Useful for UI elements that need fixed pixel sizes but relative positioning
    pub fn relative_pos_absolute_size(x: f32, y: f32, width: u32, height: u32) -> Self {
        Self {
            position_mode: ViewportMode::Relative,
            size_mode: ViewportMode::Absolute,
            x,
            y,
            width: width as f32,
            height: height as f32,
            anchor: Vec2::ZERO,
        }
    }

    /// Create a viewport with absolute position and relative size
    /// Useful for margins or offsets with scalable content
    pub fn absolute_pos_relative_size(x: u32, y: u32, width: f32, height: f32) -> Self {
        Self {
            position_mode: ViewportMode::Absolute,
            size_mode: ViewportMode::Relative,
            x: x as f32,
            y: y as f32,
            width,
            height,
            anchor: Vec2::ZERO,
        }
    }

    /// Create a centered viewport with fixed size
    pub fn centered_absolute(width: u32, height: u32) -> Self {
        Self {
            position_mode: ViewportMode::Relative,
            size_mode: ViewportMode::Absolute,
            x: 0.5,
            y: 0.5,
            width: width as f32,
            height: height as f32,
            anchor: Vec2::new(0.5, 0.5), // Center anchor
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
