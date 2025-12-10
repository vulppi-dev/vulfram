use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::render::components::{ComponentId, MeshInstance};
use crate::core::render::resources::{GeometryId, MaterialId};
use crate::core::state::EngineState;

// MARK: - Create Model

/// Arguments for creating a model component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelCreateArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
    pub geometry_id: GeometryId,
    pub material_id: MaterialId,
    pub model_mat: Mat4,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
}

impl Default for CmdModelCreateArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
            geometry_id: 0,
            material_id: 0,
            model_mat: Mat4::IDENTITY,
            layer_mask: default_layer_mask(),
        }
    }
}

/// Result for model creation command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultModelCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new model component attached to an entity
pub fn engine_cmd_model_create(
    engine: &mut EngineState,
    args: &CmdModelCreateArgs,
) -> CmdResultModelCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get or create render state
    if window_state.render_state.is_none() {
        window_state.render_state = Some(crate::core::render::RenderState::new());
    }

    let render_state = window_state.render_state.as_mut().unwrap();

    // Check if entity already has a model component
    if render_state
        .components
        .models
        .contains_key(&args.component_id)
    {
        return CmdResultModelCreate {
            success: false,
            message: format!("Entity {} already has a model component", args.component_id),
        };
    }

    // Validate geometry exists
    if !render_state
        .resources
        .geometries
        .contains_key(&args.geometry_id)
    {
        return CmdResultModelCreate {
            success: false,
            message: format!("Geometry with id {} not found", args.geometry_id),
        };
    }

    // Validate material exists
    if !render_state
        .resources
        .materials
        .contains_key(&args.material_id)
    {
        return CmdResultModelCreate {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        };
    }

    // Create model instance
    let model_instance = MeshInstance {
        geometry: args.geometry_id,
        material: args.material_id,
        model_mat: args.model_mat,
        layer_mask: args.layer_mask,
        is_dirty: true,
    };

    // Insert model component
    render_state
        .components
        .models
        .insert(args.component_id, model_instance);

    CmdResultModelCreate {
        success: true,
        message: "Model component created successfully".into(),
    }
}

// MARK: - Update Model

/// Arguments for updating a model component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelUpdateArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
    pub geometry_id: Option<GeometryId>,
    pub material_id: Option<MaterialId>,
    pub model_mat: Option<Mat4>,
    pub layer_mask: Option<u32>,
}

impl Default for CmdModelUpdateArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
            geometry_id: None,
            material_id: None,
            model_mat: None,
            layer_mask: None,
        }
    }
}

/// Result for model update command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelUpdate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultModelUpdate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Update an existing model component
pub fn engine_cmd_model_update(
    engine: &mut EngineState,
    args: &CmdModelUpdateArgs,
) -> CmdResultModelUpdate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultModelUpdate {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // Get model component
    let model = match render_state.components.models.get_mut(&args.component_id) {
        Some(m) => m,
        None => {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Entity {} has no model component", args.component_id),
            };
        }
    };

    // Update geometry if provided
    if let Some(geometry_id) = args.geometry_id {
        // Validate geometry exists
        if !render_state.resources.geometries.contains_key(&geometry_id) {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Geometry with id {} not found", geometry_id),
            };
        }
        model.geometry = geometry_id;
        model.is_dirty = true;
    }

    // Update material if provided
    if let Some(material_id) = args.material_id {
        // Validate material exists
        if !render_state.resources.materials.contains_key(&material_id) {
            return CmdResultModelUpdate {
                success: false,
                message: format!("Material with id {} not found", material_id),
            };
        }
        model.material = material_id;
        model.is_dirty = true;
    }

    // Update model matrix if provided
    if let Some(model_mat) = args.model_mat {
        model.model_mat = model_mat;
        model.is_dirty = true;
    }

    // Update layer mask if provided
    if let Some(layer_mask) = args.layer_mask {
        model.layer_mask = layer_mask;
        model.is_dirty = true;
    }

    CmdResultModelUpdate {
        success: true,
        message: "Model component updated successfully".into(),
    }
}

// MARK: - Dispose Model

/// Arguments for disposing a model component
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdModelDisposeArgs {
    pub component_id: ComponentId,
    pub window_id: u32,
}

impl Default for CmdModelDisposeArgs {
    fn default() -> Self {
        Self {
            component_id: 0,
            window_id: 0,
        }
    }
}

/// Result for model dispose command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultModelDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultModelDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a model component
pub fn engine_cmd_model_dispose(
    engine: &mut EngineState,
    args: &CmdModelDisposeArgs,
) -> CmdResultModelDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultModelDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultModelDispose {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // 🆕 Get model to find its material (and thus shader)
    let shader_id = if let Some(model) = render_state.components.models.get(&args.component_id) {
        if let Some(material) = render_state.resources.materials.get(&model.material) {
            Some(material.pipeline_spec.shader_id)
        } else {
            None
        }
    } else {
        return CmdResultModelDispose {
            success: false,
            message: format!("Entity {} has no model component", args.component_id),
        };
    };

    // 🆕 Deallocate from shader's buffer (group 2 = model)
    if let Some(shader_id) = shader_id {
        if let Some(shader) = render_state.resources.shaders.get_mut(&shader_id) {
            shader.uniform_buffers.deallocate(2, args.component_id);
        }
    }

    // 🆕 Remove bindings
    render_state
        .binding_manager
        .remove_component_bindings(args.component_id);

    // Remove model component
    render_state.components.models.remove(&args.component_id);

    CmdResultModelDispose {
        success: true,
        message: "Model component disposed successfully".into(),
    }
}

// MARK: - Helpers

fn default_layer_mask() -> u32 {
    0xFF
}
