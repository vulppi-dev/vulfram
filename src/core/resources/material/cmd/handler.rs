use super::types::*;
use super::utils::*;
use crate::core::resources::{
    MATERIAL_FALLBACK_ID, MaterialPbrParams, MaterialPbrRecord, MaterialStandardParams,
    MaterialStandardRecord,
};
use crate::core::state::EngineState;

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .materials_standard
        .contains_key(&args.material_id)
        || window_state
            .render_state
            .scene
            .materials_pbr
            .contains_key(&args.material_id)
    {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material with id {} already exists", args.material_id),
        };
    }

    if args.kind != MaterialKind::Standard && args.kind != MaterialKind::Pbr {
        return CmdResultMaterialCreate {
            success: false,
            message: "Unsupported material kind".into(),
        };
    }

    match args.kind {
        MaterialKind::Standard => {
            let opts = match &args.options {
                Some(MaterialOptions::Standard(opts)) => opts.clone(),
                None => StandardOptions::default(),
                _ => StandardOptions::default(),
            };
            if let Some(message) =
                validate_standard_texture_ids(&window_state.render_state.scene, &opts)
            {
                return CmdResultMaterialCreate {
                    success: false,
                    message,
                };
            }

            let mut record =
                MaterialStandardRecord::new(args.label.clone(), MaterialStandardParams::default());
            pack_standard_material(args.material_id, &opts, &mut record);
            record.bind_group = None;
            window_state
                .render_state
                .scene
                .materials_standard
                .insert(args.material_id, record);
        }
        MaterialKind::Pbr => {
            let opts = match &args.options {
                Some(MaterialOptions::Pbr(opts)) => opts.clone(),
                None => PbrOptions::default(),
                _ => PbrOptions::default(),
            };
            if let Some(message) = validate_pbr_texture_ids(&window_state.render_state.scene, &opts)
            {
                return CmdResultMaterialCreate {
                    success: false,
                    message,
                };
            }

            let mut record =
                MaterialPbrRecord::new(args.label.clone(), MaterialPbrParams::default());
            pack_pbr_material(args.material_id, &opts, &mut record);
            record.bind_group = None;
            window_state
                .render_state
                .scene
                .materials_pbr
                .insert(args.material_id, record);
        }
    }

    window_state.is_dirty = true;

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    let kind = args.kind.unwrap_or(MaterialKind::Standard); // Default to standard if not specified? Or check both?

    match kind {
        MaterialKind::Standard => {
            if !window_state
                .render_state
                .scene
                .materials_standard
                .contains_key(&args.material_id)
            {
                return CmdResultMaterialUpdate {
                    success: false,
                    message: format!("Material with id {} not found", args.material_id),
                };
            }
        }
        MaterialKind::Pbr => {
            if !window_state
                .render_state
                .scene
                .materials_pbr
                .contains_key(&args.material_id)
            {
                return CmdResultMaterialUpdate {
                    success: false,
                    message: format!("Material with id {} not found", args.material_id),
                };
            }
        }
    }

    if let Some(opts) = &args.options {
        match opts {
            MaterialOptions::Standard(opts) => {
                if let Some(message) =
                    validate_standard_texture_ids(&window_state.render_state.scene, &opts)
                {
                    return CmdResultMaterialUpdate {
                        success: false,
                        message,
                    };
                }
                if let Some(record) = window_state
                    .render_state
                    .scene
                    .materials_standard
                    .get_mut(&args.material_id)
                {
                    pack_standard_material(args.material_id, &opts, record);
                    record.bind_group = None;
                    record.mark_dirty();
                }
            }
            MaterialOptions::Pbr(opts) => {
                if let Some(message) =
                    validate_pbr_texture_ids(&window_state.render_state.scene, &opts)
                {
                    return CmdResultMaterialUpdate {
                        success: false,
                        message,
                    };
                }
                if let Some(record) = window_state
                    .render_state
                    .scene
                    .materials_pbr
                    .get_mut(&args.material_id)
                {
                    pack_pbr_material(args.material_id, &opts, record);
                    record.bind_group = None;
                    record.mark_dirty();
                }
            }
        }
    }

    window_state.is_dirty = true;

    CmdResultMaterialUpdate {
        success: true,
        message: "Material updated successfully".into(),
    }
}

pub fn engine_cmd_material_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDisposeArgs,
) -> CmdResultMaterialDispose {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if args.material_id == MATERIAL_FALLBACK_ID {
        return CmdResultMaterialDispose {
            success: false,
            message: "Fallback material cannot be disposed".into(),
        };
    }

    let removed_standard = window_state
        .render_state
        .scene
        .materials_standard
        .remove(&args.material_id)
        .is_some();
    let removed_pbr = window_state
        .render_state
        .scene
        .materials_pbr
        .remove(&args.material_id)
        .is_some();

    if removed_standard || removed_pbr {
        window_state.is_dirty = true;
        CmdResultMaterialDispose {
            success: true,
            message: "Material disposed successfully".into(),
        }
    } else {
        CmdResultMaterialDispose {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        }
    }
}
