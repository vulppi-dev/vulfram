use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::resources::common::default_layer_mask;
use crate::core::resources::{ComponentContainer, ModelComponent};
use crate::core::state::EngineState;

// MARK: - Create Model

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelCreateArgs {
    pub model_id: u32,
    pub transform: Mat4,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
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
    let window_states = &mut engine.window.states;

    for (_, window_state) in window_states.iter_mut() {
        if window_state
            .render_state
            .models
            .contains_key(&args.model_id)
        {
            return CmdResultModelCreate {
                success: false,
                message: format!("Model with id {} already exists", args.model_id),
            };
        }
    }

    for (_, window_state) in window_states.iter_mut() {
        let component = ModelComponent::new(args.transform);
        let container = ComponentContainer::new(component, args.layer_mask);
        window_state
            .render_state
            .models
            .insert(args.model_id, container);
        window_state.is_dirty = true;
    }

    CmdResultModelCreate {
        success: true,
        message: "Model created successfully".into(),
    }
}

// MARK: - Update Model

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelUpdateArgs {
    pub model_id: u32,
    pub transform: Option<Mat4>,
    pub layer_mask: Option<u32>,
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
    let window_states = &mut engine.window.states;

    let mut found = false;
    for (_, window_state) in window_states.iter_mut() {
        if let Some(container) = window_state.render_state.models.get_mut(&args.model_id) {
            found = true;

            container.data.update(args.transform);

            if let Some(layer_mask) = args.layer_mask {
                container.layer_mask = layer_mask;
            }

            container.mark_dirty();
            window_state.is_dirty = true;
        }
    }

    if found {
        CmdResultModelUpdate {
            success: true,
            message: "Model updated successfully".into(),
        }
    } else {
        CmdResultModelUpdate {
            success: false,
            message: format!("Model with id {} not found", args.model_id),
        }
    }
}

// MARK: - Dispose Model

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelDisposeArgs {
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
    let window_states = &mut engine.window.states;

    let mut found = false;
    for (_, window_state) in window_states.iter_mut() {
        if window_state
            .render_state
            .models
            .remove(&args.model_id)
            .is_some()
        {
            found = true;
            window_state.is_dirty = true;
        }
    }

    if found {
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
