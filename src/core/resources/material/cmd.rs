use glam::Vec4;
use serde::{Deserialize, Serialize};

use crate::core::resources::{
    MaterialStandardComponent, MaterialStandardRecord, SurfaceType, MATERIAL_FALLBACK_ID,
};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MaterialKind {
    Standard,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardOptions {
    pub base_color: Vec4,
    pub surface_type: SurfaceType,
}

impl Default for StandardOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            surface_type: SurfaceType::Opaque,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum MaterialOptions {
    Standard(StandardOptions),
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
        .materials_standard
        .contains_key(&args.material_id)
    {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material with id {} already exists", args.material_id),
        };
    }

    if args.kind != MaterialKind::Standard {
        return CmdResultMaterialCreate {
            success: false,
            message: "Unsupported material kind".into(),
        };
    }

    let opts = match &args.options {
        Some(MaterialOptions::Standard(opts)) => opts.clone(),
        None => StandardOptions::default(),
    };

    let component = MaterialStandardComponent {
        base_color: opts.base_color,
    };
    let mut record = MaterialStandardRecord::new(component);
    record.surface_type = opts.surface_type;
    window_state
        .render_state
        .scene
        .materials_standard
        .insert(args.material_id, record);

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

    if let Some(kind) = args.kind {
        if kind != MaterialKind::Standard {
            return CmdResultMaterialUpdate {
                success: false,
                message: "Unsupported material kind".into(),
            };
        }
    }

    let options = match &args.options {
        Some(MaterialOptions::Standard(opts)) => Some(opts.clone()),
        None => None,
    };

    if let Some(record) = window_state
        .render_state
        .scene
        .materials_standard
        .get_mut(&args.material_id)
    {
        if let Some(opts) = options {
            record.data.base_color = opts.base_color;
            record.surface_type = opts.surface_type;
        }
        record.mark_dirty();
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

    if window_state
        .render_state
        .scene
        .materials_standard
        .remove(&args.material_id)
        .is_some()
    {
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
