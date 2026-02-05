use serde::{Deserialize, Serialize};

use crate::core::render::graph::LogicalId;
use crate::core::state::EngineState;

use super::state::{UiContextRecord, UiThemeRecord};
use super::tree::{UiOp, UiTreeState, apply_ops};
use super::types::{UiRectPx, UiRenderTarget, UiThemeSource};

// MARK: - Theme Define

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiThemeDefineArgs {
    pub theme_id: LogicalId,
    pub source: UiThemeSource,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiThemeDefine {
    pub success: bool,
    pub message: String,
    pub theme_id: Option<LogicalId>,
    pub theme_version: Option<u32>,
}

pub fn engine_cmd_ui_theme_define(
    engine: &mut EngineState,
    args: &CmdUiThemeDefineArgs,
) -> CmdResultUiThemeDefine {
    let record = engine
        .ui
        .themes
        .entry(args.theme_id.clone())
        .or_insert_with(|| UiThemeRecord {
            theme_id: args.theme_id.clone(),
            version: 0,
            source: args.source.clone(),
        });

    record.version = record.version.saturating_add(1);
    record.source = args.source.clone();

    CmdResultUiThemeDefine {
        success: true,
        message: "Theme stored".into(),
        theme_id: Some(record.theme_id.clone()),
        theme_version: Some(record.version),
    }
}

// MARK: - Context Create

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiContextCreateArgs {
    pub window_id: u32,
    pub context_id: LogicalId,
    pub theme_id: LogicalId,
    pub target: UiRenderTarget,
    pub screen_rect: UiRectPx,
    pub z_index: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiContextCreate {
    pub success: bool,
    pub message: String,
    pub context_id: Option<LogicalId>,
}

pub fn engine_cmd_ui_context_create(
    engine: &mut EngineState,
    args: &CmdUiContextCreateArgs,
) -> CmdResultUiContextCreate {
    if engine.ui.contexts.contains_key(&args.context_id) {
        return CmdResultUiContextCreate {
            success: false,
            message: format!("UiContext {} already exists", args.context_id),
            context_id: Some(args.context_id.clone()),
        };
    }

    if !engine.ui.themes.contains_key(&args.theme_id) {
        return CmdResultUiContextCreate {
            success: false,
            message: format!("Theme {} not found", args.theme_id),
            context_id: Some(args.context_id.clone()),
        };
    }

    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultUiContextCreate {
            success: false,
            message: format!("Window {} not found", args.window_id),
            context_id: Some(args.context_id.clone()),
        };
    }

    let record = UiContextRecord {
        window_id: args.window_id,
        _context_id: args.context_id.clone(),
        theme_id: args.theme_id.clone(),
        target: args.target.clone(),
        screen_rect: args.screen_rect.clone(),
        z_index: args.z_index.unwrap_or(0),
        tree: UiTreeState::with_root(),
        render_target: None,
        egui_ctx: egui::Context::default(),
        focused_node: None,
        viewport_requests: Vec::new(),
        debug_map_logged: false,
        debug_draw_logged: false,
    };

    engine.ui.contexts.insert(args.context_id.clone(), record);

    CmdResultUiContextCreate {
        success: true,
        message: "UiContext created".into(),
        context_id: Some(args.context_id.clone()),
    }
}

// MARK: - Context Update

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiContextSetThemeArgs {
    pub context_id: LogicalId,
    pub theme_id: LogicalId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiContextSetRectArgs {
    pub context_id: LogicalId,
    pub screen_rect: UiRectPx,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiContextSetTargetArgs {
    pub context_id: LogicalId,
    pub target: UiRenderTarget,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiContextUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_context_set_theme(
    engine: &mut EngineState,
    args: &CmdUiContextSetThemeArgs,
) -> CmdResultUiContextUpdate {
    let context = match engine.ui.contexts.get_mut(&args.context_id) {
        Some(record) => record,
        None => {
            return CmdResultUiContextUpdate {
                success: false,
                message: format!("UiContext {} not found", args.context_id),
            };
        }
    };

    if !engine.ui.themes.contains_key(&args.theme_id) {
        return CmdResultUiContextUpdate {
            success: false,
            message: format!("Theme {} not found", args.theme_id),
        };
    }

    context.theme_id = args.theme_id.clone();

    CmdResultUiContextUpdate {
        success: true,
        message: "UiContext theme updated".into(),
    }
}

pub fn engine_cmd_ui_context_set_rect(
    engine: &mut EngineState,
    args: &CmdUiContextSetRectArgs,
) -> CmdResultUiContextUpdate {
    let context = match engine.ui.contexts.get_mut(&args.context_id) {
        Some(record) => record,
        None => {
            return CmdResultUiContextUpdate {
                success: false,
                message: format!("UiContext {} not found", args.context_id),
            };
        }
    };

    context.screen_rect = args.screen_rect.clone();

    CmdResultUiContextUpdate {
        success: true,
        message: "UiContext rect updated".into(),
    }
}

pub fn engine_cmd_ui_context_set_target(
    engine: &mut EngineState,
    args: &CmdUiContextSetTargetArgs,
) -> CmdResultUiContextUpdate {
    let context = match engine.ui.contexts.get_mut(&args.context_id) {
        Some(record) => record,
        None => {
            return CmdResultUiContextUpdate {
                success: false,
                message: format!("UiContext {} not found", args.context_id),
            };
        }
    };

    context.target = args.target.clone();

    CmdResultUiContextUpdate {
        success: true,
        message: "UiContext target updated".into(),
    }
}

// MARK: - Context Dispose

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiContextDisposeArgs {
    pub context_id: LogicalId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiContextDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_context_dispose(
    engine: &mut EngineState,
    args: &CmdUiContextDisposeArgs,
) -> CmdResultUiContextDispose {
    if engine.ui.contexts.remove(&args.context_id).is_none() {
        return CmdResultUiContextDispose {
            success: false,
            message: format!("UiContext {} not found", args.context_id),
        };
    }

    CmdResultUiContextDispose {
        success: true,
        message: "UiContext disposed".into(),
    }
}

// MARK: - Apply Ops

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiApplyOpsArgs {
    pub context_id: LogicalId,
    pub base_version: u32,
    pub ops: Vec<UiOp>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiApplyOps {
    pub success: bool,
    pub message: String,
    pub context_id: Option<LogicalId>,
    pub new_version: Option<u32>,
    pub expected_base_version: Option<u32>,
    pub current_version: Option<u32>,
}

pub fn engine_cmd_ui_apply_ops(
    engine: &mut EngineState,
    args: &CmdUiApplyOpsArgs,
) -> CmdResultUiApplyOps {
    let context = match engine.ui.contexts.get_mut(&args.context_id) {
        Some(record) => record,
        None => {
            return CmdResultUiApplyOps {
                success: false,
                message: format!("UiContext {} not found", args.context_id),
                context_id: Some(args.context_id.clone()),
                ..CmdResultUiApplyOps::default()
            };
        }
    };

    if args.base_version != context.tree.version {
        return CmdResultUiApplyOps {
            success: false,
            message: "UiOps rejected: baseVersion mismatch".into(),
            context_id: Some(args.context_id.clone()),
            expected_base_version: Some(context.tree.version),
            current_version: Some(context.tree.version),
            ..CmdResultUiApplyOps::default()
        };
    }

    let mut staged = context.tree.clone();
    if let Err(message) = apply_ops(&mut staged, &args.ops) {
        return CmdResultUiApplyOps {
            success: false,
            message,
            context_id: Some(args.context_id.clone()),
            ..CmdResultUiApplyOps::default()
        };
    }

    staged.version = staged.version.saturating_add(1);
    context.tree = staged;

    CmdResultUiApplyOps {
        success: true,
        message: "UiOps applied".into(),
        context_id: Some(args.context_id.clone()),
        new_version: Some(context.tree.version),
        ..CmdResultUiApplyOps::default()
    }
}
