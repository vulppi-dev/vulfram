use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::render::buffers::UniformValue;
use crate::core::render::material_types::*;
use crate::core::render::resources::{
    MaterialId, MaterialResource, PipelineSpec, ShaderId, TextureId,
};
use crate::core::state::EngineState;

// MARK: - Create Material

/// Arguments for creating a material resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialCreateArgs {
    pub material_id: MaterialId,
    pub window_id: u32,
    pub shader_id: ShaderId,
    pub textures: Vec<TextureId>,
    pub blend: Option<BlendStateDesc>,
    pub depth_stencil: Option<DepthStencilStateDesc>,
    pub primitive: PrimitiveStateDesc,
    pub label: Option<String>,
    /// Custom uniform values for material-specific parameters
    pub uniform_values: Option<HashMap<String, UniformValue>>,
}

impl Default for CmdMaterialCreateArgs {
    fn default() -> Self {
        Self {
            material_id: 0,
            window_id: 0,
            shader_id: 0,
            textures: Vec::new(),
            blend: None,
            depth_stencil: None,
            primitive: PrimitiveStateDesc::default(),
            label: None,
            uniform_values: None,
        }
    }
}

/// Result for material creation command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultMaterialCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new material resource
pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if material already exists
    if render_state
        .resources
        .materials
        .contains_key(&args.material_id)
    {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material with id {} already exists", args.material_id),
        };
    }

    // Validate shader exists
    if !render_state.resources.shaders.contains_key(&args.shader_id) {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Shader with id {} not found", args.shader_id),
        };
    }

    // Validate all textures exist
    for texture_id in &args.textures {
        if !render_state.resources.textures.contains_key(texture_id) {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Texture with id {} not found", texture_id),
            };
        }
    }

    // Convert blend state
    let blend = args.blend.as_ref().map(|b| wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: b.color.src_factor.to_wgpu(),
            dst_factor: b.color.dst_factor.to_wgpu(),
            operation: b.color.operation.to_wgpu(),
        },
        alpha: wgpu::BlendComponent {
            src_factor: b.alpha.src_factor.to_wgpu(),
            dst_factor: b.alpha.dst_factor.to_wgpu(),
            operation: b.alpha.operation.to_wgpu(),
        },
    });

    // Convert depth stencil state
    let depth_stencil = args
        .depth_stencil
        .as_ref()
        .map(|ds| wgpu::DepthStencilState {
            format: ds.format.to_wgpu(),
            depth_write_enabled: ds.depth_write_enabled,
            depth_compare: ds.depth_compare.to_wgpu(),
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: ds.stencil.front.compare.to_wgpu(),
                    fail_op: ds.stencil.front.fail_op.to_wgpu(),
                    depth_fail_op: ds.stencil.front.depth_fail_op.to_wgpu(),
                    pass_op: ds.stencil.front.pass_op.to_wgpu(),
                },
                back: wgpu::StencilFaceState {
                    compare: ds.stencil.back.compare.to_wgpu(),
                    fail_op: ds.stencil.back.fail_op.to_wgpu(),
                    depth_fail_op: ds.stencil.back.depth_fail_op.to_wgpu(),
                    pass_op: ds.stencil.back.pass_op.to_wgpu(),
                },
                read_mask: ds.stencil.read_mask,
                write_mask: ds.stencil.write_mask,
            },
            bias: wgpu::DepthBiasState {
                constant: ds.bias.constant,
                slope_scale: ds.bias.slope_scale,
                clamp: ds.bias.clamp,
            },
        });

    // Convert primitive state
    let primitive = wgpu::PrimitiveState {
        topology: args.primitive.topology.to_wgpu(),
        strip_index_format: args.primitive.strip_index_format.map(|f| f.to_wgpu()),
        front_face: args.primitive.front_face.to_wgpu(),
        cull_mode: args.primitive.cull_mode.map(|c| c.to_wgpu()),
        unclipped_depth: args.primitive.unclipped_depth,
        polygon_mode: args.primitive.polygon_mode.to_wgpu(),
        conservative: args.primitive.conservative,
    };

    // Create pipeline spec
    let pipeline_spec = PipelineSpec {
        shader_id: args.shader_id,
        blend,
        depth_stencil,
        primitive,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    };

    // Create material resource (pipeline will be created lazily on first use)
    let material_resource = MaterialResource {
        material_id: args.material_id,
        pipeline_spec,
        pipeline: None,
        textures: args.textures.clone(),
        uniform_values: args.uniform_values.clone().unwrap_or_default(),
    };

    // Insert material resource
    render_state
        .resources
        .materials
        .insert(args.material_id, material_resource);

    CmdResultMaterialCreate {
        success: true,
        message: "Material resource created successfully".into(),
    }
}

// MARK: - Update Material

/// Arguments for updating a material resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialUpdateArgs {
    pub material_id: MaterialId,
    pub window_id: u32,
    pub shader_id: Option<ShaderId>,
    pub textures: Option<Vec<TextureId>>,
}

impl Default for CmdMaterialUpdateArgs {
    fn default() -> Self {
        Self {
            material_id: 0,
            window_id: 0,
            shader_id: None,
            textures: None,
        }
    }
}

/// Result for material update command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialUpdate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultMaterialUpdate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Update an existing material resource
pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if material exists
    let material = match render_state.resources.materials.get_mut(&args.material_id) {
        Some(m) => m,
        None => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Material with id {} not found", args.material_id),
            };
        }
    };

    let mut pipeline_changed = false;

    // Update shader if provided
    if let Some(shader_id) = args.shader_id {
        // Validate shader exists
        if !render_state.resources.shaders.contains_key(&shader_id) {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Shader with id {} not found", shader_id),
            };
        }
        material.pipeline_spec.shader_id = shader_id;
        pipeline_changed = true;
    }

    // Update textures if provided
    if let Some(textures) = &args.textures {
        // Validate all textures exist
        for texture_id in textures {
            if !render_state.resources.textures.contains_key(texture_id) {
                return CmdResultMaterialUpdate {
                    success: false,
                    message: format!("Texture with id {} not found", texture_id),
                };
            }
        }
        material.textures = textures.clone();
    }

    // Invalidate pipeline if spec changed
    if pipeline_changed {
        material.pipeline = None;
    }

    CmdResultMaterialUpdate {
        success: true,
        message: "Material resource updated successfully".into(),
    }
}

// MARK: - Dispose Material

/// Arguments for disposing a material resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialDisposeArgs {
    pub material_id: MaterialId,
    pub window_id: u32,
}

impl Default for CmdMaterialDisposeArgs {
    fn default() -> Self {
        Self {
            material_id: 0,
            window_id: 0,
        }
    }
}

/// Result for material dispose command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultMaterialDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a material resource
pub fn engine_cmd_material_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDisposeArgs,
) -> CmdResultMaterialDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if material exists
    let in_use = render_state
        .components
        .models
        .values()
        .any(|m| m.material == args.material_id);

    if in_use {
        return CmdResultMaterialDispose {
            success: false,
            message: format!(
                "Material {} is still in use by one or more models",
                args.material_id
            ),
        };
    }

    // 🆕 Remove bindings and pipelines
    render_state
        .binding_manager
        .remove_material_bindings(args.material_id);
    render_state
        .pipeline_cache
        .remove_material_pipelines(args.material_id);

    // Remove material resource
    match render_state.resources.materials.remove(&args.material_id) {
        Some(_) => CmdResultMaterialDispose {
            success: true,
            message: "Material resource disposed successfully".into(),
        },
        None => CmdResultMaterialDispose {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        },
    }
}
