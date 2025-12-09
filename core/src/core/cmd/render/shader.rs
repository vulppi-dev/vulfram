use serde::{Deserialize, Serialize};

use crate::core::render::buffers::UniformBufferLayout;
use crate::core::render::resources::{
    ShaderId, ShaderResource, StorageBufferBinding, TextureBinding, UniformBufferBinding,
    VertexAttributeSpec,
};
use crate::core::state::EngineState;

// MARK: - Create Shader

/// Arguments for creating a shader resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdShaderCreateArgs {
    pub shader_id: ShaderId,
    pub window_id: u32,
    pub buffer_id: u64,
    pub label: Option<String>,

    // Shader interface metadata
    pub uniform_buffers: Vec<UniformBufferBinding>,
    pub texture_bindings: Vec<TextureBinding>,
    pub storage_buffers: Vec<StorageBufferBinding>,
    pub vertex_attributes: Vec<VertexAttributeSpec>,
}

impl Default for CmdShaderCreateArgs {
    fn default() -> Self {
        Self {
            shader_id: 0,
            window_id: 0,
            buffer_id: 0,
            label: None,
            uniform_buffers: Vec::new(),
            texture_bindings: Vec::new(),
            storage_buffers: Vec::new(),
            vertex_attributes: Vec::new(),
        }
    }
}

/// Result for shader creation command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultShaderCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultShaderCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new shader resource from uploaded buffer
pub fn engine_cmd_shader_create(
    engine: &mut EngineState,
    args: &CmdShaderCreateArgs,
) -> CmdResultShaderCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultShaderCreate {
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

    // Check if shader already exists
    if render_state.resources.shaders.contains_key(&args.shader_id) {
        return CmdResultShaderCreate {
            success: false,
            message: format!("Shader with id {} already exists", args.shader_id),
        };
    }

    // Get device
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultShaderCreate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    // Get shader source from upload buffer
    let shader_source = match engine.buffers.get(&args.buffer_id) {
        Some(buffer) => match std::str::from_utf8(&buffer.data) {
            Ok(source) => source,
            Err(_) => {
                return CmdResultShaderCreate {
                    success: false,
                    message: "Invalid UTF-8 in shader source".into(),
                };
            }
        },
        None => {
            return CmdResultShaderCreate {
                success: false,
                message: format!("Upload buffer with id {} not found", args.buffer_id),
            };
        }
    };

    // Create shader module
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: args.label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Calculate uniform buffer layouts
    let uniform_layouts: Vec<UniformBufferLayout> = args
        .uniform_buffers
        .iter()
        .map(|binding| {
            UniformBufferLayout::from_fields(binding.group, binding.binding, &binding.fields)
        })
        .collect();

    // Create shader resource with metadata (move instead of clone)
    let shader_resource = ShaderResource {
        shader_id: args.shader_id,
        module,
        uniform_layouts,
        texture_bindings: args.texture_bindings.clone(),
        storage_buffers: args.storage_buffers.clone(),
        vertex_attributes: args.vertex_attributes.clone(),
    };

    // Insert shader resource
    render_state
        .resources
        .shaders
        .insert(args.shader_id, shader_resource);

    // Remove upload buffer after use
    engine.buffers.remove(&args.buffer_id);

    CmdResultShaderCreate {
        success: true,
        message: "Shader resource created successfully".into(),
    }
}

// MARK: - Dispose Shader

/// Arguments for disposing a shader resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdShaderDisposeArgs {
    pub shader_id: ShaderId,
    pub window_id: u32,
}

impl Default for CmdShaderDisposeArgs {
    fn default() -> Self {
        Self {
            shader_id: 0,
            window_id: 0,
        }
    }
}

/// Result for shader dispose command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultShaderDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultShaderDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a shader resource
pub fn engine_cmd_shader_dispose(
    engine: &mut EngineState,
    args: &CmdShaderDisposeArgs,
) -> CmdResultShaderDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultShaderDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultShaderDispose {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // Check if shader is in use by any materials
    let in_use = render_state
        .resources
        .materials
        .values()
        .any(|m| m.pipeline_spec.shader_id == args.shader_id);

    if in_use {
        return CmdResultShaderDispose {
            success: false,
            message: format!(
                "Shader {} is still in use by one or more materials",
                args.shader_id
            ),
        };
    }

    // Remove shader resource
    match render_state.resources.shaders.remove(&args.shader_id) {
        Some(_) => CmdResultShaderDispose {
            success: true,
            message: "Shader resource disposed successfully".into(),
        },
        None => CmdResultShaderDispose {
            success: false,
            message: format!("Shader with id {} not found", args.shader_id),
        },
    }
}
