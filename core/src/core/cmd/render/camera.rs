use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::components::{CameraInstance, ComponentId, Viewport};
use crate::core::state::EngineState;

// MARK: - Create Camera

/// Arguments for creating a camera component
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraCreateArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
    pub proj_mat: Mat4,
    pub view_mat: Mat4,
    pub viewport: Viewport,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
}

impl Default for CmdCameraCreateArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
            proj_mat: Mat4::IDENTITY,
            view_mat: Mat4::IDENTITY,
            viewport: Viewport::default(),
            layer_mask: default_layer_mask(),
        }
    }
}

fn default_layer_mask() -> u32 {
    0xFFFFFFFF
}

/// Result for camera creation command
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    let render_state = &mut window_state.render_state;

    // Check if entity already has a camera component
    if render_state
        .components
        .cameras
        .contains_key(&args.component_id)
    {
        return CmdResultCameraCreate {
            success: false,
            message: format!(
                "Entity {} already has a camera component",
                args.component_id
            ),
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

    // Create viewport - use directly from args
    let viewport = args.viewport.clone();

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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

    // Create camera instance
    let camera_instance = CameraInstance {
        viewport,
        proj_mat: args.proj_mat,
        view_mat: args.view_mat,
        render_target,
        render_target_view,
        layer_mask: args.layer_mask,
        is_dirty: true,
    };

    // Insert camera component
    render_state
        .components
        .cameras
        .insert(args.component_id, camera_instance);

    // Mark window as dirty to trigger redraw
    window_state.is_dirty = true;

    CmdResultCameraCreate {
        success: true,
        message: "Camera component created successfully".into(),
    }
}

// MARK: - Update Camera

/// Arguments for updating a camera component
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraUpdateArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
    pub proj_mat: Option<Mat4>,
    pub view_mat: Option<Mat4>,
    pub viewport: Option<Viewport>,
    pub layer_mask: Option<u32>,
}

impl Default for CmdCameraUpdateArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
            proj_mat: None,
            view_mat: None,
            viewport: None,
            layer_mask: None,
        }
    }
}

/// Result for camera update command
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    let render_state = &mut window_state.render_state;

    // Get camera component
    let camera = match render_state.components.cameras.get_mut(&args.component_id) {
        Some(c) => c,
        None => {
            return CmdResultCameraUpdate {
                success: false,
                message: format!("Entity {} has no camera component", args.component_id),
            };
        }
    };

    // Update viewport if provided
    if let Some(viewport) = &args.viewport {
        camera.viewport = viewport.clone();
        camera.is_dirty = true;
    }

    // Update projection matrix if provided
    if let Some(proj_mat) = args.proj_mat {
        camera.proj_mat = proj_mat;
        camera.is_dirty = true;
    }

    // Update view matrix if provided
    if let Some(view_mat) = args.view_mat {
        camera.view_mat = view_mat;
        camera.is_dirty = true;
    }

    // Update layer mask if provided
    if let Some(layer_mask) = args.layer_mask {
        camera.layer_mask = layer_mask;
        camera.is_dirty = true;
    }

    // Mark window as dirty to trigger redraw
    window_state.is_dirty = true;

    CmdResultCameraUpdate {
        success: true,
        message: "Camera component updated successfully".into(),
    }
}

// MARK: - Dispose Camera

/// Arguments for disposing a camera component
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraDisposeArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
}

impl Default for CmdCameraDisposeArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
        }
    }
}

/// Result for camera dispose command
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    let render_state = &mut window_state.render_state;

    // 🆕 Get all shaders that have bindings with this camera
    let shader_ids = render_state
        .binding_manager
        .get_shaders_for_component(args.component_id);

    // 🆕 Deallocate from each shader's buffer (group 0 = camera)
    for shader_id in shader_ids {
        if let Some(shader) = render_state.resources.shaders.get_mut(&shader_id) {
            shader.uniform_buffers.deallocate(0, args.component_id);
        }
    }

    // 🆕 Remove bindings
    render_state
        .binding_manager
        .remove_component_bindings(args.component_id);

    // 🆕 Remove from blit bind group cache
    render_state.blit_bind_group_cache.remove(&args.component_id);

    // Remove camera component (render_target dropped automatically)
    match render_state.components.cameras.remove(&args.component_id) {
        Some(_) => CmdResultCameraDispose {
            success: true,
            message: "Camera component disposed successfully".into(),
        },
        None => CmdResultCameraDispose {
            success: false,
            message: format!("Entity {} has no camera component", args.component_id),
        },
    }
}
