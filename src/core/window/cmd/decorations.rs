use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

// MARK: - Set Decorations (Borderless)

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetDecorationsArgs {
    pub window_id: u32,
    pub decorations: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetDecorations {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_decorations(
    engine: &mut EngineState,
    args: &CmdWindowSetDecorationsArgs,
) -> CmdResultWindowSetDecorations {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_decorations(args.decorations);
            CmdResultWindowSetDecorations {
                success: true,
                message: "Window decorations set successfully".into(),
            }
        }
        None => CmdResultWindowSetDecorations {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_decorations(
    _engine: &mut EngineState,
    args: &CmdWindowSetDecorationsArgs,
) -> CmdResultWindowSetDecorations {
    CmdResultWindowSetDecorations {
        success: false,
        message: format!(
            "Window decorations are not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Has Decorations

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowHasDecorationsArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowHasDecorations {
    success: bool,
    message: String,
    content: bool,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_has_decorations(
    engine: &EngineState,
    args: &CmdWindowHasDecorationsArgs,
) -> CmdResultWindowHasDecorations {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let has_decorations = window_state.window.is_decorated();
            CmdResultWindowHasDecorations {
                success: true,
                message: "Window decorations state retrieved successfully".into(),
                content: has_decorations,
            }
        }
        None => CmdResultWindowHasDecorations {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: false,
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_has_decorations(
    _engine: &EngineState,
    args: &CmdWindowHasDecorationsArgs,
) -> CmdResultWindowHasDecorations {
    CmdResultWindowHasDecorations {
        success: false,
        message: format!(
            "Window decorations are not supported in wasm (window_id={})",
            args.window_id
        ),
        content: false,
    }
}

// MARK: - Set Resizable

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetResizableArgs {
    pub window_id: u32,
    pub resizable: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetResizable {
    success: bool,
    message: String,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_set_resizable(
    engine: &mut EngineState,
    args: &CmdWindowSetResizableArgs,
) -> CmdResultWindowSetResizable {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_resizable(args.resizable);
            CmdResultWindowSetResizable {
                success: true,
                message: "Window resizable property set successfully".into(),
            }
        }
        None => CmdResultWindowSetResizable {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_set_resizable(
    _engine: &mut EngineState,
    args: &CmdWindowSetResizableArgs,
) -> CmdResultWindowSetResizable {
    CmdResultWindowSetResizable {
        success: false,
        message: format!(
            "Window resizing is not supported in wasm (window_id={})",
            args.window_id
        ),
    }
}

// MARK: - Is Resizable

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowIsResizableArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowIsResizable {
    success: bool,
    message: String,
    content: bool,
}

#[cfg(not(feature = "wasm"))]
pub fn engine_cmd_window_is_resizable(
    engine: &EngineState,
    args: &CmdWindowIsResizableArgs,
) -> CmdResultWindowIsResizable {
    match engine.window.states.get(&args.window_id) {
        Some(window_state) => {
            let is_resizable = window_state.window.is_resizable();
            CmdResultWindowIsResizable {
                success: true,
                message: "Window resizable state retrieved successfully".into(),
                content: is_resizable,
            }
        }
        None => CmdResultWindowIsResizable {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: false,
        },
    }
}

#[cfg(feature = "wasm")]
pub fn engine_cmd_window_is_resizable(
    _engine: &EngineState,
    args: &CmdWindowIsResizableArgs,
) -> CmdResultWindowIsResizable {
    CmdResultWindowIsResizable {
        success: false,
        message: format!(
            "Window resizing is not supported in wasm (window_id={})",
            args.window_id
        ),
        content: false,
    }
}
