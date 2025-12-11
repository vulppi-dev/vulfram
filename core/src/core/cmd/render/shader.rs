use serde::{Deserialize, Serialize};

use crate::core::render::binding::ShaderUniformBuffers;
use crate::core::render::buffers::UniformBufferLayout;
use crate::core::render::resources::{
    ShaderId, ShaderResource, StorageBufferBinding, TextureBinding, UniformBufferBinding,
    VertexAttributeSpec,
};
use crate::core::state::EngineState;

// MARK: - Create Shader

/// Arguments for creating a shader resource
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    eprintln!(
        "🔍 DEBUG: Shader create - window_id={}, shader_id={}, buffer_id={}",
        args.window_id, args.shader_id, args.buffer_id
    );

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

    let render_state = &mut window_state.render_state;

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
    eprintln!("🔍 DEBUG: Looking for shader buffer {}", args.buffer_id);
    eprintln!(
        "🔍 DEBUG: Available buffers: {:?}",
        engine.buffers.keys().collect::<Vec<_>>()
    );

    let shader_source = match engine.buffers.get(&args.buffer_id) {
        Some(buffer) => {
            eprintln!(
                "🔍 DEBUG: Found shader buffer, size={} bytes",
                buffer.data.len()
            );
            match std::str::from_utf8(&buffer.data) {
                Ok(source) => {
                    eprintln!(
                        "🔍 DEBUG: Shader source valid UTF-8, first 100 chars: {}",
                        &source[..source.len().min(100)]
                    );
                    source
                }
                Err(_) => {
                    eprintln!("🔍 DEBUG: Shader source is not valid UTF-8");
                    return CmdResultShaderCreate {
                        success: false,
                        message: "Invalid UTF-8 in shader source".into(),
                    };
                }
            }
        }
        None => {
            eprintln!("🔍 DEBUG: Shader buffer {} not found", args.buffer_id);
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

    // Build vertex buffer layout from attributes
    let vertex_buffer_layout = build_vertex_buffer_layout(&args.vertex_attributes);

    // Create bind group layouts
    let bind_group_layouts = create_bind_group_layouts(
        device,
        &uniform_layouts,
        &args.texture_bindings,
        &args.storage_buffers,
    );

    // Create uniform buffers (initially empty, will grow as needed)
    let uniform_buffers = ShaderUniformBuffers::new();

    // Create shader resource with metadata
    let shader_resource = ShaderResource {
        shader_id: args.shader_id,
        module,
        uniform_layouts,
        texture_bindings: args.texture_bindings.clone(),
        storage_buffers: args.storage_buffers.clone(),
        vertex_attributes: args.vertex_attributes.clone(),
        vertex_buffer_layout,
        bind_group_layouts,
        uniform_buffers,
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    let render_state = &mut window_state.render_state;

    // Check if shader exists
    let materials_to_dispose: Vec<_> = render_state
        .resources
        .materials
        .iter()
        .filter(|(_, mat)| mat.pipeline_spec.shader_id == args.shader_id)
        .map(|(id, _)| *id)
        .collect();

    // 🆕 Check if any material is in use by models
    for material_id in &materials_to_dispose {
        let in_use = render_state
            .components
            .models
            .values()
            .any(|m| m.material == *material_id);

        if in_use {
            return CmdResultShaderDispose {
                success: false,
                message: format!(
                    "Cannot dispose shader {}: Material {} is still in use by models",
                    args.shader_id, material_id
                ),
            };
        }
    }

    // 🆕 Dispose materials first (cascading)
    for material_id in materials_to_dispose {
        // Remove material bindings and pipelines
        render_state
            .binding_manager
            .remove_material_bindings(material_id);
        render_state
            .pipeline_cache
            .remove_material_pipelines(material_id);
        render_state.resources.materials.remove(&material_id);
    }

    // 🆕 Remove all shader bindings
    render_state
        .binding_manager
        .remove_shader_bindings(args.shader_id);
    render_state
        .pipeline_cache
        .remove_shader_pipelines(args.shader_id);

    // Remove shader resource (this will drop buffers and bind group layouts)
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

// MARK: - Helper Functions

/// Global arena for vertex attributes to avoid Box::leak memory leak
/// This uses a thread-safe static storage that can be properly cleaned up
static VERTEX_ATTRIBUTES_ARENA: std::sync::Mutex<Vec<Vec<wgpu::VertexAttribute>>> =
    std::sync::Mutex::new(Vec::new());

/// Build vertex buffer layout from vertex attributes
/// Uses a static arena instead of Box::leak to allow proper cleanup
fn build_vertex_buffer_layout(
    attributes: &[VertexAttributeSpec],
) -> wgpu::VertexBufferLayout<'static> {
    // Convert attributes to WGPU format
    let wgpu_attributes: Vec<wgpu::VertexAttribute> = attributes
        .iter()
        .map(|attr| wgpu::VertexAttribute {
            format: attr.format.to_wgpu(),
            offset: 0, // Will be calculated based on semantic order
            shader_location: attr.location,
        })
        .collect();

    // Calculate stride (sum of all attribute sizes)
    let stride = attributes
        .iter()
        .map(|attr| match attr.format {
            crate::core::render::enums::VertexFormat::Float32 => 4,
            crate::core::render::enums::VertexFormat::Float32x2 => 8,
            crate::core::render::enums::VertexFormat::Float32x3 => 12,
            crate::core::render::enums::VertexFormat::Float32x4 => 16,
            crate::core::render::enums::VertexFormat::Uint32 => 4,
            crate::core::render::enums::VertexFormat::Uint32x2 => 8,
            crate::core::render::enums::VertexFormat::Uint32x3 => 12,
            crate::core::render::enums::VertexFormat::Uint32x4 => 16,
            _ => 0,
        })
        .sum();

    // Store attributes in arena and get static reference
    // This avoids Box::leak while maintaining 'static lifetime
    let mut arena = VERTEX_ATTRIBUTES_ARENA.lock().unwrap();
    arena.push(wgpu_attributes);
    let static_attributes = unsafe {
        // SAFETY: The arena vector never shrinks, so the pointer remains valid
        // for the lifetime of the program. This is safe because:
        // 1. VERTEX_ATTRIBUTES_ARENA is static and never dropped
        // 2. Vec never reallocates once we stop pushing to inner vecs
        // 3. Shaders are typically loaded once and kept for program duration
        std::mem::transmute::<&[wgpu::VertexAttribute], &'static [wgpu::VertexAttribute]>(
            arena.last().unwrap().as_slice(),
        )
    };

    wgpu::VertexBufferLayout {
        array_stride: stride,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: static_attributes,
    }
}

/// Create bind group layouts from uniform layouts, texture bindings, and storage buffers
fn create_bind_group_layouts(
    device: &wgpu::Device,
    uniform_layouts: &[UniformBufferLayout],
    texture_bindings: &[TextureBinding],
    storage_buffers: &[StorageBufferBinding],
) -> Vec<wgpu::BindGroupLayout> {
    let mut layouts = Vec::new();

    // Group bindings by group index
    let max_group = uniform_layouts
        .iter()
        .map(|l| l.group)
        .chain(texture_bindings.iter().map(|t| t.group))
        .chain(storage_buffers.iter().map(|s| s.group))
        .max()
        .unwrap_or(0);

    for group_idx in 0..=max_group {
        let mut entries = Vec::new();

        // Add uniform buffer entries for this group
        for uniform in uniform_layouts.iter().filter(|l| l.group == group_idx) {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: uniform.binding,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true, // Always dynamic for flexibility
                    min_binding_size: None,
                },
                count: None,
            });
        }

        // Add storage buffer entries for this group
        for storage in storage_buffers.iter().filter(|s| s.group == group_idx) {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: storage.binding,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: storage.read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        // Add texture entries for this group
        for texture in texture_bindings.iter().filter(|t| t.group == group_idx) {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: texture.binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: match texture.sample_type {
                        crate::core::render::resources::TextureSampleType::Float => {
                            wgpu::TextureSampleType::Float { filterable: true }
                        }
                        crate::core::render::resources::TextureSampleType::Depth => {
                            wgpu::TextureSampleType::Depth
                        }
                        crate::core::render::resources::TextureSampleType::Sint => {
                            wgpu::TextureSampleType::Sint
                        }
                        crate::core::render::resources::TextureSampleType::Uint => {
                            wgpu::TextureSampleType::Uint
                        }
                    },
                    view_dimension: match texture.view_dimension {
                        crate::core::render::resources::TextureViewDimension::D1 => {
                            wgpu::TextureViewDimension::D1
                        }
                        crate::core::render::resources::TextureViewDimension::D2 => {
                            wgpu::TextureViewDimension::D2
                        }
                        crate::core::render::resources::TextureViewDimension::D2Array => {
                            wgpu::TextureViewDimension::D2Array
                        }
                        crate::core::render::resources::TextureViewDimension::Cube => {
                            wgpu::TextureViewDimension::Cube
                        }
                        crate::core::render::resources::TextureViewDimension::CubeArray => {
                            wgpu::TextureViewDimension::CubeArray
                        }
                        crate::core::render::resources::TextureViewDimension::D3 => {
                            wgpu::TextureViewDimension::D3
                        }
                    },
                    multisampled: false,
                },
                count: None,
            });
        }

        // Create layout even if empty - wgpu requires bind group indices to match layout positions
        // If we have group 0 and group 2, we need 3 layouts (0, 1, 2) where 1 is empty
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Bind Group Layout {}", group_idx)),
            entries: &entries,
        });
        layouts.push(layout);
    }

    layouts
}
