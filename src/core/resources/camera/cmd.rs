use glam::{Mat4, Vec2};
use serde::{Deserialize, Serialize};

use crate::core::resources::common::default_layer_mask;
use crate::core::resources::{
    CameraComponent, CameraKind, CameraRecord, RenderTarget, ViewPosition,
};
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
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default)]
    pub order: i32,
    pub view_position: Option<ViewPosition>,
    #[serde(default = "default_ortho_scale")]
    pub ortho_scale: f32,
}

fn default_ortho_scale() -> f32 {
    10.0
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
            .scene
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
        let (target_width, target_height) = args
            .view_position
            .as_ref()
            .map(|vp| vp.resolve_size(window_state.config.width, window_state.config.height))
            .unwrap_or((window_state.config.width, window_state.config.height));

        let component = CameraComponent::new(
            args.transform,
            args.kind,
            args.flags,
            args.near_far,
            (target_width, target_height),
            args.ortho_scale,
        );
        let mut record = CameraRecord::new(
            component,
            args.layer_mask,
            args.order,
            args.view_position.clone(),
        );
        if let Some(device) = engine.device.as_ref() {
            let size = wgpu::Extent3d {
                width: target_width,
                height: target_height,
                depth_or_array_layers: 1,
            };
            let target = RenderTarget::new(device, size, wgpu::TextureFormat::Rgba32Float);
            record.set_render_target(target);
        }
        window_state
            .render_state
            .scene
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
    pub layer_mask: Option<u32>,
    pub order: Option<i32>,
    pub view_position: Option<ViewPosition>,
    pub ortho_scale: Option<f32>,
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
        if let Some(record) = window_state
            .render_state
            .scene
            .cameras
            .get_mut(&args.camera_id)
        {
            found = true;

            // Update view_position if provided
            if let Some(view_position) = args.view_position.clone() {
                record.view_position = Some(view_position);
            }

            // Calculate window size from view_position or use window dimensions
            let (target_width, target_height) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(window_state.config.width, window_state.config.height))
                .unwrap_or((window_state.config.width, window_state.config.height));

            // Get ortho_scale from args or use default
            let ortho_scale = args.ortho_scale.unwrap_or(10.0);

            // Update camera component
            record.data.update(
                args.transform,
                args.kind,
                args.flags,
                args.near_far,
                (target_width, target_height),
                ortho_scale,
            );

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
            .scene
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
