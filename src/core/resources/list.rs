use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceEntry {
    pub id: u32,
    pub label: Option<String>,
}

// -----------------------------------------------------------------------------
// List Models
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdModelListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelList {
    pub success: bool,
    pub message: String,
    pub models: Vec<ResourceEntry>,
}

pub fn engine_cmd_model_list(
    engine: &mut EngineState,
    args: &CmdModelListArgs,
) -> CmdResultModelList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let models = window_state
        .render_state
        .scene
        .models
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultModelList {
        success: true,
        message: "Models listed successfully".into(),
        models,
    }
}

// -----------------------------------------------------------------------------
// List Materials
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialList {
    pub success: bool,
    pub message: String,
    pub materials: Vec<ResourceEntry>,
}

pub fn engine_cmd_material_list(
    engine: &mut EngineState,
    args: &CmdMaterialListArgs,
) -> CmdResultMaterialList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let mut materials = Vec::new();

    for (&id, rec) in &window_state.render_state.scene.materials_standard {
        materials.push(ResourceEntry {
            id,
            label: rec.label.clone(),
        });
    }

    for (&id, rec) in &window_state.render_state.scene.materials_pbr {
        materials.push(ResourceEntry {
            id,
            label: rec.label.clone(),
        });
    }

    CmdResultMaterialList {
        success: true,
        message: "Materials listed successfully".into(),
        materials,
    }
}

// -----------------------------------------------------------------------------
// List Textures
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureList {
    pub success: bool,
    pub message: String,
    pub textures: Vec<ResourceEntry>,
}

pub fn engine_cmd_texture_list(
    engine: &mut EngineState,
    args: &CmdTextureListArgs,
) -> CmdResultTextureList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let mut textures = Vec::new();

    for (&id, rec) in &window_state.render_state.scene.textures {
        textures.push(ResourceEntry {
            id,
            label: rec.label.clone(),
        });
    }

    for (&id, entry) in &window_state.render_state.scene.forward_atlas_entries {
        textures.push(ResourceEntry {
            id,
            label: entry.label.clone(),
        });
    }

    CmdResultTextureList {
        success: true,
        message: "Textures listed successfully".into(),
        textures,
    }
}

// -----------------------------------------------------------------------------
// List Geometry (Added for completeness while we're at it)
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryList {
    pub success: bool,
    pub message: String,
    pub geometries: Vec<ResourceEntry>,
}

pub fn engine_cmd_geometry_list(
    engine: &mut EngineState,
    args: &CmdGeometryListArgs,
) -> CmdResultGeometryList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let geometries = if let Some(va) = &window_state.render_state.vertex {
        va.records()
            .iter()
            .map(|(&id, rec)| ResourceEntry {
                id,
                label: rec.label.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    CmdResultGeometryList {
        success: true,
        message: "Geometries listed successfully".into(),
        geometries,
    }
}

// -----------------------------------------------------------------------------
// List Lights
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightList {
    pub success: bool,
    pub message: String,
    pub lights: Vec<ResourceEntry>,
}

pub fn engine_cmd_light_list(
    engine: &mut EngineState,
    args: &CmdLightListArgs,
) -> CmdResultLightList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultLightList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let lights = window_state
        .render_state
        .scene
        .lights
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultLightList {
        success: true,
        message: "Lights listed successfully".into(),
        lights,
    }
}

// -----------------------------------------------------------------------------
// List Cameras
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCameraListArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultCameraList {
    pub success: bool,
    pub message: String,
    pub cameras: Vec<ResourceEntry>,
}

pub fn engine_cmd_camera_list(
    engine: &mut EngineState,
    args: &CmdCameraListArgs,
) -> CmdResultCameraList {
    let window_state = match engine.window.states.get(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultCameraList {
                success: false,
                message: format!("Window {} not found", args.window_id),
                ..Default::default()
            };
        }
    };

    let cameras = window_state
        .render_state
        .scene
        .cameras
        .iter()
        .map(|(&id, rec)| ResourceEntry {
            id,
            label: rec.label.clone(),
        })
        .collect();

    CmdResultCameraList {
        success: true,
        message: "Cameras listed successfully".into(),
        cameras,
    }
}
