use glam::Vec4;
use serde::{Deserialize, Serialize};

use crate::core::resources::{
    MaterialLambertComponent, MaterialLambertRecord, MaterialUnlitComponent,
    MaterialUnlitRecord, MATERIAL_FALLBACK_ID,
};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MaterialKind {
    Unlit,
    Lambert,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnlitOptions {
    pub base_color: Vec4,
}

impl Default for UnlitOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LambertOptions {
    pub base_color: Vec4,
}

impl Default for LambertOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum MaterialOptions {
    Unlit(UnlitOptions),
    Lambert(LambertOptions),
}

// MARK: - Create Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialCreateArgs {
    pub window_id: u32,
    pub material_id: u32,
    pub kind: MaterialKind,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialCreate {
    pub success: bool,
    pub message: String,
}

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
        .materials_unlit
        .contains_key(&args.material_id)
        || window_state
            .render_state
            .scene
            .materials_lambert
            .contains_key(&args.material_id)
    {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material with id {} already exists", args.material_id),
        };
    }

    let options = match (args.kind, &args.options) {
        (MaterialKind::Unlit, Some(MaterialOptions::Unlit(opts))) => {
            MaterialOptions::Unlit(opts.clone())
        }
        (MaterialKind::Unlit, None) => MaterialOptions::Unlit(UnlitOptions::default()),
        (MaterialKind::Lambert, Some(MaterialOptions::Lambert(opts))) => {
            MaterialOptions::Lambert(opts.clone())
        }
        (MaterialKind::Lambert, None) => MaterialOptions::Lambert(LambertOptions::default()),
        (kind, Some(_)) => {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Options type mismatch for {:?}", kind),
            };
        }
    };

    match options {
        MaterialOptions::Unlit(opts) => {
            let component = MaterialUnlitComponent {
                base_color: opts.base_color,
            };
            let record = MaterialUnlitRecord::new(component);
            window_state
                .render_state
                .scene
                .materials_unlit
                .insert(args.material_id, record);
        }
        MaterialOptions::Lambert(opts) => {
            let component = MaterialLambertComponent {
                base_color: opts.base_color,
            };
            let record = MaterialLambertRecord::new(component);
            window_state
                .render_state
                .scene
                .materials_lambert
                .insert(args.material_id, record);
        }
    }

    window_state.is_dirty = true;

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

// MARK: - Update Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialUpdateArgs {
    pub window_id: u32,
    pub material_id: u32,
    pub kind: Option<MaterialKind>,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialUpdate {
    pub success: bool,
    pub message: String,
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

    let existing_kind = if window_state
        .render_state
        .scene
        .materials_unlit
        .contains_key(&args.material_id)
    {
        Some(MaterialKind::Unlit)
    } else if window_state
        .render_state
        .scene
        .materials_lambert
        .contains_key(&args.material_id)
    {
        Some(MaterialKind::Lambert)
    } else {
        None
    };

    let existing_kind = match existing_kind {
        Some(kind) => kind,
        None => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Material with id {} not found", args.material_id),
            };
        }
    };

    let target_kind = args.kind.unwrap_or(existing_kind);

    let options = match (target_kind, &args.options) {
        (MaterialKind::Unlit, Some(MaterialOptions::Unlit(opts))) => {
            Some(MaterialOptions::Unlit(opts.clone()))
        }
        (MaterialKind::Lambert, Some(MaterialOptions::Lambert(opts))) => {
            Some(MaterialOptions::Lambert(opts.clone()))
        }
        (MaterialKind::Unlit, None) | (MaterialKind::Lambert, None) => None,
        (kind, Some(_)) => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Options type mismatch for {:?}", kind),
            };
        }
    };

    if existing_kind != target_kind {
        match existing_kind {
            MaterialKind::Unlit => {
                window_state
                    .render_state
                    .scene
                    .materials_unlit
                    .remove(&args.material_id);
            }
            MaterialKind::Lambert => {
                window_state
                    .render_state
                    .scene
                    .materials_lambert
                    .remove(&args.material_id);
            }
        }

        match target_kind {
            MaterialKind::Unlit => {
                let opts = match options {
                    Some(MaterialOptions::Unlit(opts)) => opts,
                    _ => UnlitOptions::default(),
                };
                let component = MaterialUnlitComponent {
                    base_color: opts.base_color,
                };
                let record = MaterialUnlitRecord::new(component);
                window_state
                    .render_state
                    .scene
                    .materials_unlit
                    .insert(args.material_id, record);
            }
            MaterialKind::Lambert => {
                let opts = match options {
                    Some(MaterialOptions::Lambert(opts)) => opts,
                    _ => LambertOptions::default(),
                };
                let component = MaterialLambertComponent {
                    base_color: opts.base_color,
                };
                let record = MaterialLambertRecord::new(component);
                window_state
                    .render_state
                    .scene
                    .materials_lambert
                    .insert(args.material_id, record);
            }
        }

        window_state.is_dirty = true;

        return CmdResultMaterialUpdate {
            success: true,
            message: "Material updated successfully".into(),
        };
    }

    match target_kind {
        MaterialKind::Unlit => {
            if let Some(record) = window_state
                .render_state
                .scene
                .materials_unlit
                .get_mut(&args.material_id)
            {
                if let Some(MaterialOptions::Unlit(opts)) = options {
                    record.data.base_color = opts.base_color;
                }
                record.mark_dirty();
            }
        }
        MaterialKind::Lambert => {
            if let Some(record) = window_state
                .render_state
                .scene
                .materials_lambert
                .get_mut(&args.material_id)
            {
                if let Some(MaterialOptions::Lambert(opts)) = options {
                    record.data.base_color = opts.base_color;
                }
                record.mark_dirty();
            }
        }
    }

    window_state.is_dirty = true;

    CmdResultMaterialUpdate {
        success: true,
        message: "Material updated successfully".into(),
    }
}

// MARK: - Dispose Material

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialDisposeArgs {
    pub window_id: u32,
    pub material_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialDispose {
    pub success: bool,
    pub message: String,
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

    let removed_unlit = window_state
        .render_state
        .scene
        .materials_unlit
        .remove(&args.material_id)
        .is_some();
    let removed_lambert = window_state
        .render_state
        .scene
        .materials_lambert
        .remove(&args.material_id)
        .is_some();

    if removed_unlit || removed_lambert {
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
