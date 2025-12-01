use serde::{Deserialize, Serialize};

use crate::core::units::IVector2;

/// Window-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum WindowEvent {
    /// Window was created successfully
    OnCreate { window_id: u32 },

    /// Window was resized
    OnResize {
        window_id: u32,
        width: u32,
        height: u32,
    },

    /// Window was moved
    OnMove { window_id: u32, position: IVector2 },

    /// Window close was requested by user
    OnCloseRequest { window_id: u32 },

    /// Window was destroyed
    OnDestroy { window_id: u32 },

    /// Window gained or lost focus
    OnFocus { window_id: u32, focused: bool },

    /// Window scale factor changed
    OnScaleFactorChange {
        window_id: u32,
        scale_factor: f64,
        new_width: u32,
        new_height: u32,
    },

    /// Window was occluded (completely hidden from view)
    OnOcclude { window_id: u32, occluded: bool },

    /// Window redraw was requested
    OnRedrawRequest { window_id: u32 },

    /// File was dropped into window
    OnFileDrop { window_id: u32, path: String },

    /// File is being hovered over window
    OnFileHover { window_id: u32, path: String },

    /// Hovered file left the window
    OnFileHoverCancel { window_id: u32 },

    /// System theme changed
    OnThemeChange { window_id: u32, dark_mode: bool },
}
