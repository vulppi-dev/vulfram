use glam::{Mat4, Vec2, Vec4};
use serde::{Deserialize, Serialize};

use crate::core::resources::common::default_layer_mask;
use crate::core::resources::{CameraComponent, CameraKind, CameraRecord, RenderTarget, ViewPosition};
use crate::core::state::EngineState;

// MARK: - Create Camera

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraCreateArgs {
    pub camera_id: u32,
    pub transform: Mat4,
    pub kind: CameraKind,
    #[serde(default)]
    pub flags: u32,
    pub near_far: Vec2,
    pub viewport: Vec4,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default)]
    pub order: i32,
    pub view_position: Option<ViewPosition>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_camera_create(
    engine: &mut EngineState,
    args: &CmdCameraCreateArgs,
) -> CmdResultCameraCreate {
    let window_states = &mut engine.window.states;

    for (_, window_state) in window_states.iter_mut() {
        if window_state
            .render_state
            .cameras
            .contains_key(&args.camera_id)
        {
            return CmdResultCameraCreate {
                success: false,
                message: format!("Camera with id {} already exists", args.camera_id),
            };
        }
    }

    for (_, window_state) in window_states.iter_mut() {
        let component = CameraComponent::new(
            args.transform,
            args.kind,
            args.flags,
            args.near_far,
            args.viewport,
        );
        let mut record =
            CameraRecord::new(component, args.layer_mask, args.order, args.view_position.clone());
        if let Some(device) = engine.device.as_ref() {
            let (target_width, target_height) = args
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(window_state.config.width, window_state.config.height))
                .unwrap_or((window_state.config.width, window_state.config.height));
            let size = wgpu::Extent3d {
                width: target_width,
                height: target_height,
                depth_or_array_layers: 1,
            };
            let target = RenderTarget::new(device, size, window_state.config.format);
            record.set_render_target(target);
        }
        window_state
            .render_state
            .cameras
            .insert(args.camera_id, record);
        window_state.is_dirty = true;
    }

    CmdResultCameraCreate {
        success: true,
        message: "Camera created successfully".into(),
    }
}

// MARK: - Update Camera

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraUpdateArgs {
    pub camera_id: u32,
    pub transform: Option<Mat4>,
    pub kind: Option<CameraKind>,
    pub flags: Option<u32>,
    pub near_far: Option<Vec2>,
    pub viewport: Option<Vec4>,
    pub layer_mask: Option<u32>,
    pub order: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_camera_update(
    engine: &mut EngineState,
    args: &CmdCameraUpdateArgs,
) -> CmdResultCameraUpdate {
    let window_states = &mut engine.window.states;

    let mut found = false;
    for (_, window_state) in window_states.iter_mut() {
        if let Some(record) = window_state.render_state.cameras.get_mut(&args.camera_id) {
            found = true;

            if let Some(viewport) = args.viewport {
                record.data.update(
                    args.transform,
                    args.kind,
                    args.flags,
                    args.near_far,
                    viewport,
                );
            }

            if let Some(layer_mask) = args.layer_mask {
                record.layer_mask = layer_mask;
            }

            if let Some(order) = args.order {
                record.order = order;
            }

            record.mark_dirty();
            window_state.is_dirty = true;
        }
    }

    if found {
        CmdResultCameraUpdate {
            success: true,
            message: "Camera updated successfully".into(),
        }
    } else {
        CmdResultCameraUpdate {
            success: false,
            message: format!("Camera with id {} not found", args.camera_id),
        }
    }
}

// MARK: - Dispose Camera

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdCameraDisposeArgs {
    pub camera_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_camera_dispose(
    engine: &mut EngineState,
    args: &CmdCameraDisposeArgs,
) -> CmdResultCameraDispose {
    let window_states = &mut engine.window.states;

    let mut found = false;
    for (_, window_state) in window_states.iter_mut() {
        if window_state
            .render_state
            .cameras
            .remove(&args.camera_id)
            .is_some()
        {
            found = true;
            window_state.is_dirty = true;
        }
    }

    if found {
        CmdResultCameraDispose {
            success: true,
            message: "Camera disposed successfully".into(),
        }
    } else {
        CmdResultCameraDispose {
            success: false,
            message: format!("Camera with id {} not found", args.camera_id),
        }
    }
}
