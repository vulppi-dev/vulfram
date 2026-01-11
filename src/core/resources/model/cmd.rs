use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::resources::common::default_layer_mask;
use crate::core::resources::{ModelComponent, ModelRecord};
use crate::core::state::EngineState;

// MARK: - Create Model

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelCreateArgs {
    pub window_id: u32,
    pub model_id: u32,
    pub label: Option<String>,
    pub geometry_id: u32,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub transform: Mat4,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub receive_shadow: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_model_create(
    engine: &mut EngineState,
    args: &CmdModelCreateArgs,
) -> CmdResultModelCreate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .models
        .contains_key(&args.model_id)
    {
        return CmdResultModelCreate {
            success: false,
            message: format!("Model with id {} already exists", args.model_id),
        };
    }

    let vertex_allocator = match window_state.render_state.vertex.as_mut() {
        Some(va) => va,
        None => {
            return CmdResultModelCreate {
                success: false,
                message: format!(
                    "Vertex allocator not initialized for window {}",
                    args.window_id
                ),
            };
        }
    };

    if vertex_allocator.vertex_count(args.geometry_id).is_err() {
        return CmdResultModelCreate {
            success: false,
            message: format!("Geometry {} not found", args.geometry_id),
        };
    }

    let component = ModelComponent::new(args.transform, args.receive_shadow);
    let record = ModelRecord::new(
        args.label.clone(),
        component,
        args.geometry_id,
        args.material_id,
        args.layer_mask,
        args.cast_shadow,
        args.receive_shadow,
    );
    window_state
        .render_state
        .scene
        .models
        .insert(args.model_id, record);
    if let Some(shadow) = window_state.render_state.shadow.as_mut() {
        shadow.mark_dirty();
    }
    window_state.is_dirty = true;

    CmdResultModelCreate {
        success: true,
        message: "Model created successfully".into(),
    }
}

// MARK: - Update Model

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelUpdateArgs {
    pub window_id: u32,
    pub model_id: u32,
    pub label: Option<String>,
    pub geometry_id: Option<u32>,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub transform: Option<Mat4>,
    pub layer_mask: Option<u32>,
    pub cast_shadow: Option<bool>,
    pub receive_shadow: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_model_update(
    engine: &mut EngineState,
    args: &CmdModelUpdateArgs,
) -> CmdResultModelUpdate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    let record = match window_state
        .render_state
        .scene
        .models
        .get_mut(&args.model_id)
    {
        Some(r) => r,
        None => {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Model with id {} not found", args.model_id),
            };
        }
    };

    if args.label.is_some() {
        record.label = args.label.clone();
    }

    if let Some(geometry_id) = args.geometry_id {
        let vertex_allocator = match window_state.render_state.vertex.as_mut() {
            Some(va) => va,
            None => {
                return CmdResultModelUpdate {
                    success: false,
                    message: format!(
                        "Vertex allocator not initialized for window {}",
                        args.window_id
                    ),
                };
            }
        };

        if vertex_allocator.vertex_count(geometry_id).is_err() {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Geometry {} not found", geometry_id),
            };
        }

        record.geometry_id = geometry_id;
    }

    if args.material_id.is_some() {
        record.material_id = args.material_id;
    }

    if let Some(cast_shadow) = args.cast_shadow {
        record.cast_shadow = cast_shadow;
    }

    if let Some(receive_shadow) = args.receive_shadow {
        record.receive_shadow = receive_shadow;
    }

    record.data.update(args.transform, args.receive_shadow);

    if let Some(layer_mask) = args.layer_mask {
        record.layer_mask = layer_mask;
    }

    record.mark_dirty();
    if let Some(shadow) = window_state.render_state.shadow.as_mut() {
        shadow.mark_dirty();
    }
    window_state.is_dirty = true;

    CmdResultModelUpdate {
        success: true,
        message: "Model updated successfully".into(),
    }
}

// MARK: - Dispose Model

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelDisposeArgs {
    pub window_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_model_dispose(
    engine: &mut EngineState,
    args: &CmdModelDisposeArgs,
) -> CmdResultModelDispose {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .models
        .remove(&args.model_id)
        .is_some()
    {
        if let Some(shadow) = window_state.render_state.shadow.as_mut() {
            shadow.mark_dirty();
        }
        window_state.is_dirty = true;
        CmdResultModelDispose {
            success: true,
            message: "Model disposed successfully".into(),
        }
    } else {
        CmdResultModelDispose {
            success: false,
            message: format!("Model with id {} not found", args.model_id),
        }
    }
}
