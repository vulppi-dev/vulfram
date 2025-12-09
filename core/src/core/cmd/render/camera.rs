use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::components::{CameraInstance, ComponentId, Viewport};
use crate::core::state::EngineState;

// MARK: - Create Camera

/// Arguments for creating a camera component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraCreateArgs {
    pub entity_id: ComponentId,
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
            entity_id: 0,
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

    // Allocate uniform buffer space for camera data
    // Camera uniforms: view_mat (64 bytes) + proj_mat (64 bytes) = 128 bytes
    let camera_uniform_offset = render_state
        .uniform_buffer_manager
        .allocate_camera(device, 128);

    // Prepare camera uniform data (view and projection matrices)
    // Pack matrices as raw bytes using bytemuck
    let mut camera_data = Vec::with_capacity(128);
    camera_data.extend_from_slice(bytemuck::bytes_of(&args.view_mat));
    camera_data.extend_from_slice(bytemuck::bytes_of(&args.proj_mat));

    // Write camera data to GPU
    let queue = match &engine.queue {
        Some(q) => q,
        None => {
            return CmdResultCameraCreate {
                success: false,
                message: "GPU queue not initialized".into(),
            };
        }
    };
    render_state.uniform_buffer_manager.write_camera_data(
        queue,
        camera_uniform_offset,
        &camera_data,
    );

    // Create camera instance
    let camera_instance = CameraInstance {
        camera_uniform_offset,
        viewport,
        proj_mat: args.proj_mat,
        view_mat: args.view_mat,
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
    pub entity_id: ComponentId,
    pub window_id: u32,
    pub proj_mat: Option<Mat4>,
    pub view_mat: Option<Mat4>,
    pub viewport: Option<Viewport>,
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
    if let Some(viewport) = &args.viewport {
        camera.viewport = viewport.clone();
    }

    // Update projection matrix if provided
    if let Some(proj_mat) = args.proj_mat {
        camera.proj_mat = proj_mat;

        // Write updated proj_mat to GPU uniform buffer
        let queue = match &engine.queue {
            Some(q) => q,
            None => {
                return CmdResultCameraUpdate {
                    success: false,
                    message: "GPU queue not initialized".into(),
                };
            }
        };

        // Projection matrix is at offset 64 within camera uniform
        let proj_offset = camera.camera_uniform_offset + 64;
        render_state.uniform_buffer_manager.write_camera_data(
            queue,
            proj_offset,
            bytemuck::bytes_of(&proj_mat),
        );
    }

    // Update view matrix if provided
    if let Some(view_mat) = args.view_mat {
        camera.view_mat = view_mat;

        // Write updated view_mat to GPU uniform buffer
        let queue = match &engine.queue {
            Some(q) => q,
            None => {
                return CmdResultCameraUpdate {
                    success: false,
                    message: "GPU queue not initialized".into(),
                };
            }
        };

        // View matrix is at offset 0 within camera uniform
        render_state.uniform_buffer_manager.write_camera_data(
            queue,
            camera.camera_uniform_offset,
            bytemuck::bytes_of(&view_mat),
        );
    }

    // Update layer mask if provided
    if let Some(layer_mask) = args.layer_mask {
        camera.layer_mask = layer_mask;
    }

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
    pub entity_id: ComponentId,
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
