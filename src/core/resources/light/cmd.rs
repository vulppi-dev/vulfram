use glam::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

use crate::core::resources::common::default_layer_mask;
use crate::core::resources::{LightComponent, LightKind, LightRecord};
use crate::core::state::EngineState;

// MARK: - Create Light

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightCreateArgs {
    pub window_id: u32,
    pub light_id: u32,
    pub label: Option<String>,
    #[serde(default)]
    pub kind: Option<LightKind>,
    #[serde(default)]
    pub position: Option<Vec4>,
    #[serde(default)]
    pub direction: Option<Vec4>,
    #[serde(default)]
    pub color: Option<Vec4>,
    #[serde(default)]
    pub ground_color: Option<Vec4>,
    #[serde(default)]
    pub intensity: Option<f32>,
    #[serde(default)]
    pub range: Option<f32>,
    #[serde(default)]
    pub spot_inner_outer: Option<Vec2>,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_light_create(
    engine: &mut EngineState,
    args: &CmdLightCreateArgs,
) -> CmdResultLightCreate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultLightCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .lights
        .contains_key(&args.light_id)
    {
        return CmdResultLightCreate {
            success: false,
            message: format!("Light with id {} already exists", args.light_id),
        };
    }

    let kind = args.kind.unwrap_or(LightKind::Point);
    let position = args.position.unwrap_or(Vec4::new(0.0, 1.0, 0.0, 1.0));
    let direction = args.direction.unwrap_or(Vec4::new(0.0, -1.0, 0.0, 0.0));
    let color = args.color.unwrap_or(Vec4::new(1.0, 1.0, 1.0, 1.0));
    let ground_color = args.ground_color.unwrap_or(Vec4::new(0.0, 0.0, 0.0, 1.0));
    let intensity = args.intensity.unwrap_or(1.0);
    let range = args.range.unwrap_or(10.0);
    let spot_inner_outer = args.spot_inner_outer.unwrap_or(Vec2::new(0.5, 0.8));

    let component = LightComponent::new(
        position,
        direction,
        color,
        ground_color,
        intensity,
        range,
        spot_inner_outer,
        kind,
        args.cast_shadow,
    );

    let record = LightRecord::new(
        args.label.clone(),
        component,
        args.layer_mask,
        args.cast_shadow,
    );
    window_state
        .render_state
        .scene
        .lights
        .insert(args.light_id, record);
    if let Some(shadow) = window_state.render_state.shadow.as_mut() {
        shadow.mark_dirty();
    }
    window_state.is_dirty = true;

    CmdResultLightCreate {
        success: true,
        message: "Light created successfully".into(),
    }
}

// MARK: - Update Light

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightUpdateArgs {
    pub window_id: u32,
    pub light_id: u32,
    pub kind: Option<LightKind>,
    pub position: Option<Vec4>,
    pub direction: Option<Vec4>,
    pub color: Option<Vec4>,
    pub ground_color: Option<Vec4>,
    pub intensity: Option<f32>,
    pub range: Option<f32>,
    pub spot_inner_outer: Option<Vec2>,
    pub layer_mask: Option<u32>,
    pub cast_shadow: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_light_update(
    engine: &mut EngineState,
    args: &CmdLightUpdateArgs,
) -> CmdResultLightUpdate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultLightUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    let record = match window_state
        .render_state
        .scene
        .lights
        .get_mut(&args.light_id)
    {
        Some(record) => record,
        None => {
            return CmdResultLightUpdate {
                success: false,
                message: format!("Light with id {} not found", args.light_id),
            };
        }
    };

    if let Some(kind) = args.kind {
        record.data.kind_flags.x = kind as u32;
    }

    if let Some(cast_shadow) = args.cast_shadow {
        record.cast_shadow = cast_shadow;
        if cast_shadow {
            record.data.kind_flags.y |= LightComponent::FLAG_CAST_SHADOW;
        } else {
            record.data.kind_flags.y &= !LightComponent::FLAG_CAST_SHADOW;
        }
    }

    if let Some(position) = args.position {
        record.data.position = position;
    }

    if let Some(direction) = args.direction {
        record.data.direction = direction;
    }

    if let Some(color) = args.color {
        record.data.color = color;
    }

    if let Some(ground_color) = args.ground_color {
        record.data.ground_color = ground_color;
    }

    if let Some(intensity) = args.intensity {
        record.data.intensity_range.x = intensity;
    }

    if let Some(range) = args.range {
        record.data.intensity_range.y = range;
    }

    if let Some(spot_inner_outer) = args.spot_inner_outer {
        record.data.spot_inner_outer = spot_inner_outer;
    }

    if let Some(layer_mask) = args.layer_mask {
        record.layer_mask = layer_mask;
    }

    record.mark_dirty();
    if let Some(shadow) = window_state.render_state.shadow.as_mut() {
        shadow.mark_dirty();
    }
    window_state.is_dirty = true;

    CmdResultLightUpdate {
        success: true,
        message: "Light updated successfully".into(),
    }
}

// MARK: - Dispose Light

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdLightDisposeArgs {
    pub window_id: u32,
    pub light_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_light_dispose(
    engine: &mut EngineState,
    args: &CmdLightDisposeArgs,
) -> CmdResultLightDispose {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultLightDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .lights
        .remove(&args.light_id)
        .is_some()
    {
        if let Some(shadow) = window_state.render_state.shadow.as_mut() {
            shadow.free_light(args.light_id);
            shadow.mark_dirty();
        }
        window_state.is_dirty = true;
        CmdResultLightDispose {
            success: true,
            message: "Light disposed successfully".into(),
        }
    } else {
        CmdResultLightDispose {
            success: false,
            message: format!("Light with id {} not found", args.light_id),
        }
    }
}
