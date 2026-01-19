use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize};

#[cfg(not(feature = "wasm"))]
use crate::core::buffers::state::UploadType;
#[cfg(not(feature = "wasm"))]
use crate::core::image::ImageDecoder;
use crate::core::state::EngineState;

use super::EngineWindowState;

// MARK: - Set Title

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetTitleArgs {
    pub window_id: u32,
    pub title: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetTitle {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_title(
    engine: &mut EngineState,
    args: &CmdWindowSetTitleArgs,
) -> CmdResultWindowSetTitle {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_title(&args.title);
            CmdResultWindowSetTitle {
                success: true,
                message: "Title set successfully".into(),
            }
        }
        None => CmdResultWindowSetTitle {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_title(
    _engine: &mut EngineState,
    args: &CmdWindowSetTitleArgs,
) -> CmdResultWindowSetTitle {
    CmdResultWindowSetTitle {
        success: false,
        message: format!(
            "Window title is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Set Position

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetPositionArgs {
    pub window_id: u32,
    pub position: IVec2,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetPosition {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_position(
    engine: &mut EngineState,
    args: &CmdWindowSetPositionArgs,
) -> CmdResultWindowSetPosition {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let position = PhysicalPosition::new(args.position[0], args.position[1]);
            window_state.window.set_outer_position(position);
            CmdResultWindowSetPosition {
                success: true,
                message: "Position set successfully".into(),
            }
        }
        None => CmdResultWindowSetPosition {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_position(
    _engine: &mut EngineState,
    args: &CmdWindowSetPositionArgs,
) -> CmdResultWindowSetPosition {
    CmdResultWindowSetPosition {
        success: false,
        message: format!(
            "Window position is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Get Position

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetPositionArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetPosition {
    success: bool,
    message: String,
    content: IVec2,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_get_position(
    engine: &EngineState,
    args: &CmdWindowGetPositionArgs,
) -> CmdResultWindowGetPosition {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => match window_state.window.outer_position() {
            Ok(position) => CmdResultWindowGetPosition {
                success: true,
                message: "Position retrieved successfully".into(),
                content: IVec2::new(position.x, position.y),
            },
            Err(e) => CmdResultWindowGetPosition {
                success: false,
                message: format!("Failed to get position: {:?}", e),
                content: IVec2::new(0, 0),
            },
        },
        None => CmdResultWindowGetPosition {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: IVec2::new(0, 0),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_get_position(
    _engine: &EngineState,
    args: &CmdWindowGetPositionArgs,
) -> CmdResultWindowGetPosition {
    CmdResultWindowGetPosition {
        success: false,
        message: format!(
            "Window position is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: IVec2::new(0, 0),
    }
}

// MARK: - Set Size

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetSizeArgs {
    pub window_id: u32,
    pub size: UVec2,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetSize {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_size(
    engine: &mut EngineState,
    args: &CmdWindowSetSizeArgs,
) -> CmdResultWindowSetSize {
    match engine.window.states.get_mut(&args.window_id) {
        Some(window_state) => {
            let size = PhysicalSize::new(args.size[0], args.size[1]);
            let _ = window_state.window.request_inner_size(size);

            // Update surface configuration
            window_state.config.width = args.size[0];
            window_state.config.height = args.size[1];
            window_state
                .surface
                .configure(engine.device.as_ref().unwrap(), &window_state.config);

            // Mark window as dirty to trigger redraw
            window_state.is_dirty = true;

            CmdResultWindowSetSize {
                success: true,
                message: "Size set successfully".into(),
            }
        }
        None => CmdResultWindowSetSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_size(
    _engine: &mut EngineState,
    args: &CmdWindowSetSizeArgs,
) -> CmdResultWindowSetSize {
    CmdResultWindowSetSize {
        success: false,
        message: format!(
            "Window size is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Get Size

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetSize {
    success: bool,
    message: String,
    content: UVec2,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_get_size(
    engine: &EngineState,
    args: &CmdWindowGetSizeArgs,
) -> CmdResultWindowGetSize {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let size = window_state.window.inner_size();
            CmdResultWindowGetSize {
                success: true,
                message: "Size retrieved successfully".into(),
                content: UVec2::new(size.width, size.height),
            }
        }
        None => CmdResultWindowGetSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_get_size(
    _engine: &EngineState,
    args: &CmdWindowGetSizeArgs,
) -> CmdResultWindowGetSize {
    CmdResultWindowGetSize {
        success: false,
        message: format!(
            "Window size is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: UVec2::new(0, 0),
    }
}

// MARK: - Get Outer Size

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetOuterSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetOuterSize {
    success: bool,
    message: String,
    content: UVec2,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_get_outer_size(
    engine: &EngineState,
    args: &CmdWindowGetOuterSizeArgs,
) -> CmdResultWindowGetOuterSize {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let size = window_state.window.outer_size();
            CmdResultWindowGetOuterSize {
                success: true,
                message: "Outer size retrieved successfully".into(),
                content: UVec2::new(size.width, size.height),
            }
        }
        None => CmdResultWindowGetOuterSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_get_outer_size(
    _engine: &EngineState,
    args: &CmdWindowGetOuterSizeArgs,
) -> CmdResultWindowGetOuterSize {
    CmdResultWindowGetOuterSize {
        success: false,
        message: format!(
            "Window outer size is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: UVec2::new(0, 0),
    }
}

// MARK: - Get Surface Size

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetSurfaceSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetSurfaceSize {
    success: bool,
    message: String,
    content: UVec2,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_get_surface_size(
    engine: &EngineState,
    args: &CmdWindowGetSurfaceSizeArgs,
) -> CmdResultWindowGetSurfaceSize {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => CmdResultWindowGetSurfaceSize {
            success: true,
            message: "Surface size retrieved successfully".into(),
            content: UVec2::new(window_state.config.width, window_state.config.height),
        },
        None => CmdResultWindowGetSurfaceSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_get_surface_size(
    _engine: &EngineState,
    args: &CmdWindowGetSurfaceSizeArgs,
) -> CmdResultWindowGetSurfaceSize {
    CmdResultWindowGetSurfaceSize {
        success: false,
        message: format!(
            "Window surface size is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: UVec2::new(0, 0),
    }
}

// MARK: - Set State

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetStateArgs {
    pub window_id: u32,
    pub state: EngineWindowState,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetState {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_state(
    engine: &mut EngineState,
    args: &CmdWindowSetStateArgs,
) -> CmdResultWindowSetState {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            match args.state {
                EngineWindowState::Minimized => {
                    window_state.window.set_minimized(true);
                }
                EngineWindowState::Maximized => {
                    window_state.window.set_maximized(true);
                }
                EngineWindowState::Windowed => {
                    window_state.window.set_minimized(false);
                    window_state.window.set_maximized(false);
                    window_state.window.set_fullscreen(None);
                }
                EngineWindowState::Fullscreen => {
                    // Exclusive fullscreen - requires video mode
                    if let Some(monitor) = window_state.window.current_monitor() {
                        if let Some(video_mode) = monitor.video_modes().next() {
                            let fullscreen = Some(winit::window::Fullscreen::Exclusive(video_mode));
                            window_state.window.set_fullscreen(fullscreen);
                        }
                    }
                }
                EngineWindowState::WindowedFullscreen => {
                    // Borderless fullscreen
                    let monitor = window_state.window.current_monitor();
                    let fullscreen = Some(winit::window::Fullscreen::Borderless(monitor));
                    window_state.window.set_fullscreen(fullscreen);
                }
            }
            CmdResultWindowSetState {
                success: true,
                message: "Window state set successfully".into(),
            }
        }
        None => CmdResultWindowSetState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_state(
    _engine: &mut EngineState,
    args: &CmdWindowSetStateArgs,
) -> CmdResultWindowSetState {
    CmdResultWindowSetState {
        success: false,
        message: format!(
            "Window state is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Get State

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetStateArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetState {
    success: bool,
    message: String,
    content: EngineWindowState,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_get_state(
    engine: &EngineState,
    args: &CmdWindowGetStateArgs,
) -> CmdResultWindowGetState {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let state = if window_state.window.is_minimized().unwrap_or(false) {
                EngineWindowState::Minimized
            } else if window_state.window.is_maximized() {
                EngineWindowState::Maximized
            } else if window_state.window.fullscreen().is_some() {
                // Check fullscreen type
                match window_state.window.fullscreen() {
                    Some(winit::window::Fullscreen::Exclusive(_)) => EngineWindowState::Fullscreen,
                    Some(winit::window::Fullscreen::Borderless(_)) => {
                        EngineWindowState::WindowedFullscreen
                    }
                    None => EngineWindowState::Windowed,
                }
            } else {
                EngineWindowState::Windowed
            };

            CmdResultWindowGetState {
                success: true,
                message: "Window state retrieved successfully".into(),
                content: state,
            }
        }
        None => CmdResultWindowGetState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: EngineWindowState::default(),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_get_state(
    _engine: &EngineState,
    args: &CmdWindowGetStateArgs,
) -> CmdResultWindowGetState {
    CmdResultWindowGetState {
        success: false,
        message: format!(
            "Window state is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: EngineWindowState::default(),
    }
}

// MARK: - Set Icon

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetIconArgs {
    pub window_id: u32,
    pub buffer_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetIcon {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_icon(
    engine: &mut EngineState,
    args: &CmdWindowSetIconArgs,
) -> CmdResultWindowSetIcon {
    // Check if window exists
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultWindowSetIcon {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    }

    // Get and remove buffer (one-shot consumption)
    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultWindowSetIcon {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
            };
        }
    };

    // Validate buffer type
    if buffer.upload_type != UploadType::ImageData {
        return CmdResultWindowSetIcon {
            success: false,
            message: format!(
                "Invalid buffer type. Expected ImageData, got {:?}",
                buffer.upload_type
            ),
        };
    }

    // Decode image from buffer data (happens when consumed)
    let image_buffer = match ImageDecoder::try_decode(&buffer.data) {
        Some(img) => img,
        None => {
            return CmdResultWindowSetIcon {
                success: false,
                message: "Failed to decode image. Supported formats: PNG, JPEG, WebP, AVIF".into(),
            };
        }
    };

    // Create winit icon (requires RGBA8 format)
    let icon = match winit::window::Icon::from_rgba(
        image_buffer.data.clone(),
        image_buffer.width,
        image_buffer.height,
    ) {
        Ok(icon) => icon,
        Err(e) => {
            return CmdResultWindowSetIcon {
                success: false,
                message: format!("Failed to create icon: {:?}", e),
            };
        }
    };

    // Apply icon to window
    let window_state = engine.window.states.get(&args.window_id).unwrap();
    window_state.window.set_window_icon(Some(icon));

    CmdResultWindowSetIcon {
        success: true,
        message: "Icon set successfully".into(),
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_icon(
    _engine: &mut EngineState,
    args: &CmdWindowSetIconArgs,
) -> CmdResultWindowSetIcon {
    CmdResultWindowSetIcon {
        success: false,
        message: format!(
            "Window icon is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}
