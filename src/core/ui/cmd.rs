use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::render::graph::LogicalId;
use crate::core::state::EngineState;

use super::animation::{parse_animation_easing, parse_animation_property, UiAnimation};
use super::state::{UiContextRecord, UiPanelRecord, UiThemeRecord};
use super::tree::{UiOp, UiTreeState, apply_ops};
use super::types::{UiRectPx, UiRenderTarget, UiThemeConfig};

// MARK: - Theme Define

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiThemeDefineArgs {
    pub theme_id: LogicalId,
    pub theme: UiThemeConfig,
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
            theme: args.theme.clone(),
        });

    record.version = record.version.saturating_add(1);
    record.theme = args.theme.clone();

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
    pub theme_id: Option<LogicalId>,
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

    if let Some(theme_id) = args.theme_id.as_ref() {
        if !engine.ui.themes.contains_key(theme_id) {
            return CmdResultUiContextCreate {
                success: false,
                message: format!("Theme {} not found", theme_id),
                context_id: Some(args.context_id.clone()),
            };
        }
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
        style_cache: HashMap::new(),
        ordered_children_cache: HashMap::new(),
        animations: Vec::new(),
        animated_overrides: HashMap::new(),
        node_rects: HashMap::new(),
        debug_enabled: false,
        applied_theme_version: 0,
        applied_theme_id: None,
        applied_theme_fallback: false,
        debug_map_logged: false,
        debug_draw_logged: false,
        first_render: true,
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
    pub theme_id: Option<LogicalId>,
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

    if let Some(theme_id) = args.theme_id.as_ref() {
        if !engine.ui.themes.contains_key(theme_id) {
            return CmdResultUiContextUpdate {
                success: false,
                message: format!("Theme {} not found", theme_id),
            };
        }
    }

    context.theme_id = args.theme_id.clone();
    context.applied_theme_id = None;
    context.applied_theme_version = 0;
    context.applied_theme_fallback = false;

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

    engine
        .ui
        .panels
        .retain(|_, panel| panel.context_id != args.context_id);

    CmdResultUiContextDispose {
        success: true,
        message: "UiContext disposed".into(),
    }
}

// MARK: - Panel Create

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiPanelCreateArgs {
    pub panel_id: LogicalId,
    pub context_id: LogicalId,
    pub model_id: u32,
    pub camera_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiPanelCreate {
    pub success: bool,
    pub message: String,
    pub panel_id: Option<LogicalId>,
}

pub fn engine_cmd_ui_panel_create(
    engine: &mut EngineState,
    args: &CmdUiPanelCreateArgs,
) -> CmdResultUiPanelCreate {
    if engine.ui.panels.contains_key(&args.panel_id) {
        return CmdResultUiPanelCreate {
            success: false,
            message: format!("UiPanel {} already exists", args.panel_id),
            panel_id: Some(args.panel_id.clone()),
        };
    }

    let context = match engine.ui.contexts.get(&args.context_id) {
        Some(context) => context,
        None => {
            return CmdResultUiPanelCreate {
                success: false,
                message: format!("UiContext {} not found", args.context_id),
                panel_id: Some(args.panel_id.clone()),
            };
        }
    };

    let window_state = match engine.window.states.get(&context.window_id) {
        Some(state) => state,
        None => {
            return CmdResultUiPanelCreate {
                success: false,
                message: format!("Window {} not found", context.window_id),
                panel_id: Some(args.panel_id.clone()),
            };
        }
    };

    if !window_state
        .render_state
        .scene
        .models
        .contains_key(&args.model_id)
    {
        return CmdResultUiPanelCreate {
            success: false,
            message: format!("Model {} not found", args.model_id),
            panel_id: Some(args.panel_id.clone()),
        };
    }

    if !window_state
        .render_state
        .scene
        .cameras
        .contains_key(&args.camera_id)
    {
        return CmdResultUiPanelCreate {
            success: false,
            message: format!("Camera {} not found", args.camera_id),
            panel_id: Some(args.panel_id.clone()),
        };
    }

    let record = UiPanelRecord {
        context_id: args.context_id.clone(),
        model_id: args.model_id,
        camera_id: args.camera_id,
        window_id: context.window_id,
    };

    engine.ui.panels.insert(args.panel_id.clone(), record);

    CmdResultUiPanelCreate {
        success: true,
        message: "UiPanel created".into(),
        panel_id: Some(args.panel_id.clone()),
    }
}

// MARK: - Panel Update

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiPanelUpdateArgs {
    pub panel_id: LogicalId,
    pub context_id: Option<LogicalId>,
    pub model_id: Option<u32>,
    pub camera_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiPanelUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_panel_update(
    engine: &mut EngineState,
    args: &CmdUiPanelUpdateArgs,
) -> CmdResultUiPanelUpdate {
    let panel = match engine.ui.panels.get_mut(&args.panel_id) {
        Some(panel) => panel,
        None => {
            return CmdResultUiPanelUpdate {
                success: false,
                message: format!("UiPanel {} not found", args.panel_id),
            };
        }
    };

    let context_id = args.context_id.as_ref().unwrap_or(&panel.context_id);
    let context = match engine.ui.contexts.get(context_id) {
        Some(context) => context,
        None => {
            return CmdResultUiPanelUpdate {
                success: false,
                message: format!("UiContext {} not found", context_id),
            };
        }
    };

    let window_state = match engine.window.states.get(&context.window_id) {
        Some(state) => state,
        None => {
            return CmdResultUiPanelUpdate {
                success: false,
                message: format!("Window {} not found", context.window_id),
            };
        }
    };

    let model_id = args.model_id.unwrap_or(panel.model_id);
    if !window_state
        .render_state
        .scene
        .models
        .contains_key(&model_id)
    {
        return CmdResultUiPanelUpdate {
            success: false,
            message: format!("Model {} not found", model_id),
        };
    }

    let camera_id = args.camera_id.unwrap_or(panel.camera_id);
    if !window_state
        .render_state
        .scene
        .cameras
        .contains_key(&camera_id)
    {
        return CmdResultUiPanelUpdate {
            success: false,
            message: format!("Camera {} not found", camera_id),
        };
    }

    panel.context_id = context_id.clone();
    panel.model_id = model_id;
    panel.camera_id = camera_id;
    panel.window_id = context.window_id;

    CmdResultUiPanelUpdate {
        success: true,
        message: "UiPanel updated".into(),
    }
}

// MARK: - Panel Dispose

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiPanelDisposeArgs {
    pub panel_id: LogicalId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiPanelDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_ui_panel_dispose(
    engine: &mut EngineState,
    args: &CmdUiPanelDisposeArgs,
) -> CmdResultUiPanelDispose {
    if engine.ui.panels.remove(&args.panel_id).is_none() {
        return CmdResultUiPanelDispose {
            success: false,
            message: format!("UiPanel {} not found", args.panel_id),
        };
    }

    CmdResultUiPanelDispose {
        success: true,
        message: "UiPanel disposed".into(),
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

    let removed_ids = collect_removed_ids(&context.tree, &args.ops);
    let mut staged = context.tree.clone();
    let mut animations_to_add = Vec::new();
    for op in &args.ops {
        match op {
            UiOp::Animate(payload) => animations_to_add.push(payload.clone()),
            _ => {
                if let Err(message) = apply_ops(&mut staged, std::slice::from_ref(op)) {
                    return CmdResultUiApplyOps {
                        success: false,
                        message,
                        context_id: Some(args.context_id.clone()),
                        ..CmdResultUiApplyOps::default()
                    };
                }
            }
        }
    }

    staged.version = staged.version.saturating_add(1);
    context.tree = staged;
    if !removed_ids.is_empty() {
        for id in &removed_ids {
            context.style_cache.remove(id);
            context.ordered_children_cache.remove(id);
            context.animated_overrides.remove(id);
            context.node_rects.remove(id);
        }
        context.animations.retain(|anim| !removed_ids.contains(&anim.node_id));
        context
            .tree
            .dirty_structure
            .retain(|id| !removed_ids.contains(id));
    }
    for anim in animations_to_add {
        let Some(property) = parse_animation_property(&anim.property) else {
            return CmdResultUiApplyOps {
                success: false,
                message: format!("Unsupported animation property {}", anim.property),
                context_id: Some(args.context_id.clone()),
                ..CmdResultUiApplyOps::default()
            };
        };
        let from = anim
            .from
            .unwrap_or_else(|| resolve_animation_from(&context.tree, &anim.id, &property));
        let easing = parse_animation_easing(anim.easing.as_deref());
        context.animations.push(UiAnimation {
            node_id: anim.id.clone(),
            property,
            from,
            to: anim.to,
            duration_ms: anim.duration_ms.max(1),
            delay_ms: anim.delay_ms.unwrap_or(0),
            easing,
            start_time: None,
            completed: false,
        });
    }

    CmdResultUiApplyOps {
        success: true,
        message: "UiOps applied".into(),
        context_id: Some(args.context_id.clone()),
        new_version: Some(context.tree.version),
        ..CmdResultUiApplyOps::default()
    }
}

fn resolve_animation_from(
    tree: &UiTreeState,
    node_id: &LogicalId,
    property: &super::animation::UiAnimationProperty,
) -> f32 {
    let Some(node) = tree.nodes.get(node_id) else {
        return match property {
            super::animation::UiAnimationProperty::Opacity => 1.0,
            super::animation::UiAnimationProperty::TranslateY => 0.0,
        };
    };
    let Some(style) = node.style.as_ref() else {
        return match property {
            super::animation::UiAnimationProperty::Opacity => 1.0,
            super::animation::UiAnimationProperty::TranslateY => 0.0,
        };
    };
    match property {
        super::animation::UiAnimationProperty::Opacity => style
            .get("opacity")
            .and_then(|value| match value {
                super::types::UiValue::Float(value) => Some(*value as f32),
                super::types::UiValue::Int(value) => Some(*value as f32),
                _ => None,
            })
            .unwrap_or(1.0),
        super::animation::UiAnimationProperty::TranslateY => style
            .get("translateY")
            .and_then(|value| match value {
                super::types::UiValue::Float(value) => Some(*value as f32),
                super::types::UiValue::Int(value) => Some(*value as f32),
                _ => None,
            })
            .unwrap_or(0.0),
    }
}

fn collect_removed_ids(tree: &UiTreeState, ops: &[UiOp]) -> std::collections::HashSet<LogicalId> {
    let mut removed = std::collections::HashSet::new();
    for op in ops {
        match op {
            UiOp::Remove(payload) => {
                collect_subtree(tree, &payload.id, &mut removed);
            }
            UiOp::Clear(payload) => {
                if let Some(node) = tree.nodes.get(&payload.id) {
                    for child in &node.children {
                        collect_subtree(tree, child, &mut removed);
                    }
                }
            }
            _ => {}
        }
    }
    removed
}

fn collect_subtree(
    tree: &UiTreeState,
    node_id: &LogicalId,
    removed: &mut std::collections::HashSet<LogicalId>,
) {
    if !removed.insert(node_id.clone()) {
        return;
    }
    if let Some(node) = tree.nodes.get(node_id) {
        for child in &node.children {
            collect_subtree(tree, child, removed);
        }
    }
}
