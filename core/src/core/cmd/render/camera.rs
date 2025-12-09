use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::components::{CameraInstance, EntityId, Viewport};
use crate::core::state::EngineState;

// MARK: - Create Camera

/// Arguments for creating a camera component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraCreateArgs {
    pub entity_id: EntityId,
    pub window_id: u32,
    pub proj_mat: Mat4,
    pub view_mat: Mat4,
    pub viewport: ViewportDesc,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
}

impl Default for CmdCameraCreateArgs {
    fn default() -> Self {
        Self {
            entity_id: 0,
            window_id: 0,
            proj_mat: Mat4::IDENTITY,
            view_mat: Mat4::IDENTITY,
            viewport: ViewportDesc::default(),
            layer_mask: default_layer_mask(),
        }
    }
}

/// Viewport descriptor for camera creation
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ViewportDesc {
    pub mode: ViewportModeDesc,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for ViewportDesc {
    fn default() -> Self {
        Self {
            mode: ViewportModeDesc::Relative,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

/// Viewport mode descriptor for serialization
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ViewportModeDesc {
    Relative,
    Absolute,
}

/// Result for camera creation command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultCameraCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new camera component attached to an entity
pub fn engine_cmd_camera_create(
    engine: &mut EngineState,
    args: &CmdCameraCreateArgs,
) -> CmdResultCameraCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultCameraCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get or create render state
    if window_state.render_state.is_none() {
        window_state.render_state = Some(crate::core::render::RenderState::new());
    }

    let render_state = window_state.render_state.as_mut().unwrap();

    // Check if entity already has a camera component
    if render_state
        .components
        .cameras
        .contains_key(&args.entity_id)
    {
        return CmdResultCameraCreate {
            success: false,
            message: format!("Entity {} already has a camera component", args.entity_id),
        };
    }

    // Get device and queue
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultCameraCreate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    // Convert viewport descriptor to viewport
    let viewport = match args.viewport.mode {
        ViewportModeDesc::Relative => Viewport::Relative {
            x: args.viewport.x as f32,
            y: args.viewport.y as f32,
            width: args.viewport.width as f32,
            height: args.viewport.height as f32,
        },
        ViewportModeDesc::Absolute => Viewport::Absolute {
            x: args.viewport.x as u32,
            y: args.viewport.y as u32,
            width: args.viewport.width as u32,
            height: args.viewport.height as u32,
        },
    };

    // Create render target texture
    let texture_size = wgpu::Extent3d {
        width: window_state.config.width,
        height: window_state.config.height,
        depth_or_array_layers: 1,
    };

    let render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: window_state.config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

    // Create camera instance
    let camera_instance = CameraInstance {
        camera_uniform_offset: 0, // TODO: Allocate from uniform buffer manager
        viewport,
        render_target,
        render_target_view,
        layer_mask: args.layer_mask,
    };

    // Insert camera component
    render_state
        .components
        .cameras
        .insert(args.entity_id, camera_instance);

    CmdResultCameraCreate {
        success: true,
        message: "Camera component created successfully".into(),
    }
}

// MARK: - Update Camera

/// Arguments for updating a camera component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraUpdateArgs {
    pub entity_id: EntityId,
    pub window_id: u32,
    pub proj_mat: Option<Mat4>,
    pub view_mat: Option<Mat4>,
    pub viewport: Option<ViewportDesc>,
    pub layer_mask: Option<u32>,
}

impl Default for CmdCameraUpdateArgs {
    fn default() -> Self {
        Self {
            entity_id: 0,
            window_id: 0,
            proj_mat: None,
            view_mat: None,
            viewport: None,
            layer_mask: None,
        }
    }
}

/// Result for camera update command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraUpdate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultCameraUpdate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Update an existing camera component
pub fn engine_cmd_camera_update(
    engine: &mut EngineState,
    args: &CmdCameraUpdateArgs,
) -> CmdResultCameraUpdate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultCameraUpdate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultCameraUpdate {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // Get camera component
    let camera = match render_state.components.cameras.get_mut(&args.entity_id) {
        Some(c) => c,
        None => {
            return CmdResultCameraUpdate {
                success: false,
                message: format!("Entity {} has no camera component", args.entity_id),
            };
        }
    };

    // Update viewport if provided
    if let Some(viewport_desc) = &args.viewport {
        camera.viewport = match viewport_desc.mode {
            ViewportModeDesc::Relative => Viewport::Relative {
                x: viewport_desc.x as f32,
                y: viewport_desc.y as f32,
                width: viewport_desc.width as f32,
                height: viewport_desc.height as f32,
            },
            ViewportModeDesc::Absolute => Viewport::Absolute {
                x: viewport_desc.x as u32,
                y: viewport_desc.y as u32,
                width: viewport_desc.width as u32,
                height: viewport_desc.height as u32,
            },
        };
    }

    // Update layer mask if provided
    if let Some(layer_mask) = args.layer_mask {
        camera.layer_mask = layer_mask;
    }

    // TODO: Update projection and view matrices in uniform buffer
    // This will be implemented when uniform buffer manager is added

    CmdResultCameraUpdate {
        success: true,
        message: "Camera component updated successfully".into(),
    }
}

// MARK: - Dispose Camera

/// Arguments for disposing a camera component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraDisposeArgs {
    pub entity_id: EntityId,
    pub window_id: u32,
}

impl Default for CmdCameraDisposeArgs {
    fn default() -> Self {
        Self {
            entity_id: 0,
            window_id: 0,
        }
    }
}

/// Result for camera dispose command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultCameraDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a camera component and free its GPU resources
pub fn engine_cmd_camera_dispose(
    engine: &mut EngineState,
    args: &CmdCameraDisposeArgs,
) -> CmdResultCameraDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultCameraDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultCameraDispose {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // Remove camera component
    match render_state.components.cameras.remove(&args.entity_id) {
        Some(_) => CmdResultCameraDispose {
            success: true,
            message: "Camera component disposed successfully".into(),
        },
        None => CmdResultCameraDispose {
            success: false,
            message: format!("Entity {} has no camera component", args.entity_id),
        },
    }
}

// MARK: - Helpers

fn default_layer_mask() -> u32 {
    0xFF
}
