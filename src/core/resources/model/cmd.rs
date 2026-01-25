use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::state::SkinningSystem;
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

// MARK: - Pose Update (Skinning)

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPoseUpdateArgs {
    pub window_id: u32,
    pub model_id: u32,
    pub bone_count: u32,
    pub matrices_buffer_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultPoseUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_pose_update(
    engine: &mut EngineState,
    args: &CmdPoseUpdateArgs,
) -> CmdResultPoseUpdate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultPoseUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    let buffer = match engine.buffers.uploads.get(&args.matrices_buffer_id) {
        Some(buffer) => buffer,
        None => {
            return CmdResultPoseUpdate {
                success: false,
                message: format!("Buffer {} not found", args.matrices_buffer_id),
            };
        }
    };

    let fail = |engine: &mut EngineState, message: String| -> CmdResultPoseUpdate {
        engine.buffers.remove_upload(args.matrices_buffer_id);
        CmdResultPoseUpdate {
            success: false,
            message,
        }
    };

    if args.bone_count == 0 {
        return fail(engine, "boneCount must be greater than 0".into());
    }

    if args.bone_count > SkinningSystem::MAX_BONES_PER_MODEL {
        return fail(
            engine,
            format!(
                "boneCount exceeds limit (max {})",
                SkinningSystem::MAX_BONES_PER_MODEL
            ),
        );
    }

    let record = match window_state
        .render_state
        .scene
        .models
        .get_mut(&args.model_id)
    {
        Some(r) => r,
        None => {
            return fail(engine, format!("Model with id {} not found", args.model_id));
        }
    };

    let vertex_allocator = match window_state.render_state.vertex.as_ref() {
        Some(va) => va,
        None => {
            return fail(engine, "Vertex allocator not initialized".into());
        }
    };

    let has_skin_streams = vertex_allocator
        .geometry_has_streams(
            record.geometry_id,
            &[
                crate::core::resources::vertex::VertexStream::Joints,
                crate::core::resources::vertex::VertexStream::Weights,
            ],
        )
        .unwrap_or(false);

    if !has_skin_streams {
        return fail(
            engine,
            "Geometry does not include SkinJoints/SkinWeights".into(),
        );
    }

    let expected_bytes = args.bone_count as usize * std::mem::size_of::<glam::Mat4>();
    if buffer.data.len() != expected_bytes {
        return fail(
            engine,
            format!(
                "Bone matrix buffer size mismatch (expected {} bytes, got {})",
                expected_bytes,
                buffer.data.len()
            ),
        );
    }

    let matrices: &[glam::Mat4] = bytemuck::cast_slice(&buffer.data);
    let bindings = match window_state.render_state.bindings.as_mut() {
        Some(bindings) => bindings,
        None => {
            return fail(engine, "Bindings not initialized".into());
        }
    };

    let allocation = window_state
        .render_state
        .skinning
        .ensure_allocation(args.model_id, args.bone_count);

    bindings.bones_pool.write_slice(allocation.offset, matrices);

    record.data.set_skinning(allocation.offset, args.bone_count);
    record.mark_dirty();
    window_state.is_dirty = true;

    engine.buffers.remove_upload(args.matrices_buffer_id);

    CmdResultPoseUpdate {
        success: true,
        message: "Pose updated successfully".into(),
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
        window_state.render_state.skinning.release(args.model_id);
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
