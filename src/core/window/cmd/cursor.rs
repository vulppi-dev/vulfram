use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::core::platform::winit;
use crate::core::state::EngineState;

// MARK: - Set Cursor Visible

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorVisibleArgs {
    pub window_id: u32,
    pub visible: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorVisible {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_visible(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorVisibleArgs,
) -> CmdResultWindowSetCursorVisible {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_cursor_visible(args.visible);
            CmdResultWindowSetCursorVisible {
                success: true,
                message: "Cursor visibility set successfully".into(),
            }
        }
        None => CmdResultWindowSetCursorVisible {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Set Cursor Grab

/// Cursor grab modes
#[repr(u32)]
#[derive(Debug, Deserialize_repr, Serialize_repr, Clone, Copy)]
pub enum CursorGrabMode {
    /// No grabbing
    None = 0,
    /// Cursor confined to window
    Confined,
    /// Cursor locked to window
    Locked,
}

impl Default for CursorGrabMode {
    fn default() -> Self {
        CursorGrabMode::None
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorGrabArgs {
    pub window_id: u32,
    pub mode: CursorGrabMode,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorGrab {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_grab(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorGrabArgs,
) -> CmdResultWindowSetCursorGrab {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let grab_mode = match args.mode {
                CursorGrabMode::None => {
                    // Release grab
                    match window_state
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                    {
                        Ok(_) => {
                            return CmdResultWindowSetCursorGrab {
                                success: true,
                                message: "Cursor grab released successfully".into(),
                            };
                        }
                        Err(e) => {
                            return CmdResultWindowSetCursorGrab {
                                success: false,
                                message: format!("Failed to release cursor grab: {}", e),
                            };
                        }
                    }
                }
                CursorGrabMode::Confined => winit::window::CursorGrabMode::Confined,
                CursorGrabMode::Locked => winit::window::CursorGrabMode::Locked,
            };

            match window_state.window.set_cursor_grab(grab_mode) {
                Ok(_) => CmdResultWindowSetCursorGrab {
                    success: true,
                    message: "Cursor grab mode set successfully".into(),
                },
                Err(e) => CmdResultWindowSetCursorGrab {
                    success: false,
                    message: format!("Failed to set cursor grab mode: {}", e),
                },
            }
        }
        None => CmdResultWindowSetCursorGrab {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Set Cursor Icon

/// Cursor icon types
#[repr(u32)]
#[derive(Debug, Deserialize_repr, Serialize_repr, Clone, Copy)]
pub enum CursorIcon {
    /// Default cursor
    Default = 0,
    /// Context menu cursor
    ContextMenu,
    /// Help cursor
    Help,
    /// Pointer/hand cursor
    Pointer,
    /// Progress cursor
    Progress,
    /// Wait cursor
    Wait,
    /// Cell cursor
    Cell,
    /// Crosshair cursor
    Crosshair,
    /// Text cursor
    Text,
    /// Vertical text cursor
    VerticalText,
    /// Alias cursor
    Alias,
    /// Copy cursor
    Copy,
    /// Move cursor
    Move,
    /// No drop cursor
    NoDrop,
    /// Not allowed cursor
    NotAllowed,
    /// Grab cursor
    Grab,
    /// Grabbing cursor
    Grabbing,
    /// E-resize cursor
    EResize,
    /// N-resize cursor
    NResize,
    /// NE-resize cursor
    NeResize,
    /// NW-resize cursor
    NwResize,
    /// S-resize cursor
    SResize,
    /// SE-resize cursor
    SeResize,
    /// SW-resize cursor
    SwResize,
    /// W-resize cursor
    WResize,
    /// EW-resize cursor
    EwResize,
    /// NS-resize cursor
    NsResize,
    /// NESW-resize cursor
    NeswResize,
    /// NWSE-resize cursor
    NwseResize,
    /// Column resize cursor
    ColResize,
    /// Row resize cursor
    RowResize,
    /// All scroll cursor
    AllScroll,
    /// Zoom in cursor
    ZoomIn,
    /// Zoom out cursor
    ZoomOut,
}

impl Default for CursorIcon {
    fn default() -> Self {
        CursorIcon::Default
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorIconArgs {
    pub window_id: u32,
    pub icon: CursorIcon,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorIcon {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_icon(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorIconArgs,
) -> CmdResultWindowSetCursorIcon {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let winit_icon = match args.icon {
                CursorIcon::Default => winit::window::CursorIcon::Default,
                CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
                CursorIcon::Help => winit::window::CursorIcon::Help,
                CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                CursorIcon::Progress => winit::window::CursorIcon::Progress,
                CursorIcon::Wait => winit::window::CursorIcon::Wait,
                CursorIcon::Cell => winit::window::CursorIcon::Cell,
                CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
                CursorIcon::Text => winit::window::CursorIcon::Text,
                CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
                CursorIcon::Alias => winit::window::CursorIcon::Alias,
                CursorIcon::Copy => winit::window::CursorIcon::Copy,
                CursorIcon::Move => winit::window::CursorIcon::Move,
                CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
                CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
                CursorIcon::Grab => winit::window::CursorIcon::Grab,
                CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                CursorIcon::EResize => winit::window::CursorIcon::EResize,
                CursorIcon::NResize => winit::window::CursorIcon::NResize,
                CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
                CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
                CursorIcon::SResize => winit::window::CursorIcon::SResize,
                CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
                CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
                CursorIcon::WResize => winit::window::CursorIcon::WResize,
                CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
                CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
                CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
                CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
                CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
                CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
                CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
                CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
                CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
            };

            window_state.window.set_cursor(winit_icon);
            CmdResultWindowSetCursorIcon {
                success: true,
                message: "Cursor icon set successfully".into(),
            }
        }
        None => CmdResultWindowSetCursorIcon {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}
