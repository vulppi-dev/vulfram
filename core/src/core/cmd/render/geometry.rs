use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::core::render::enums::IndexFormat;
use crate::core::render::resources::{GeometryId, GeometryResource, VertexAttributeDesc};
use crate::core::state::EngineState;

// MARK: - Create Geometry

/// Arguments for creating a geometry resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdGeometryCreateArgs {
    pub geometry_id: GeometryId,
    pub window_id: u32,
    pub vertex_buffer_id: u64,
    pub index_buffer_id: u64,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_attributes: Vec<VertexAttributeDesc>,
    pub index_format: IndexFormat,
    pub label: Option<String>,
}

impl Default for CmdGeometryCreateArgs {
    fn default() -> Self {
        Self {
            geometry_id: 0,
            window_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_count: 0,
            index_count: 0,
            vertex_attributes: Vec::new(),
            index_format: IndexFormat::Uint16,
            label: None,
        }
    }
}

/// Result for geometry creation command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultGeometryCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new geometry resource from uploaded buffers
pub fn engine_cmd_geometry_create(
    engine: &mut EngineState,
    args: &CmdGeometryCreateArgs,
) -> CmdResultGeometryCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if geometry already exists
    if render_state
        .resources
        .geometries
        .contains_key(&args.geometry_id)
    {
        return CmdResultGeometryCreate {
            success: false,
            message: format!("Geometry with id {} already exists", args.geometry_id),
        };
    }

    // Get device
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    // Get vertex data from upload buffer
    let vertex_data = match engine.buffers.get(&args.vertex_buffer_id) {
        Some(buffer) => &buffer.data,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: format!(
                    "Vertex upload buffer with id {} not found",
                    args.vertex_buffer_id
                ),
            };
        }
    };

    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: args.label.as_deref(),
        contents: vertex_data,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    // Get index data from upload buffer
    let index_data = match engine.buffers.get(&args.index_buffer_id) {
        Some(buffer) => &buffer.data,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: format!(
                    "Index upload buffer with id {} not found",
                    args.index_buffer_id
                ),
            };
        }
    };

    // Create index buffer
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: args.label.as_deref(),
        contents: index_data,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });

    // Calculate vertex stride from attributes
    let vertex_stride = if args.vertex_attributes.is_empty() {
        0
    } else {
        args.vertex_attributes
            .iter()
            .map(|attr| {
                let size = match attr.format {
                    crate::core::render::enums::VertexFormat::Float32 => 4,
                    crate::core::render::enums::VertexFormat::Float32x2 => 8,
                    crate::core::render::enums::VertexFormat::Float32x3 => 12,
                    crate::core::render::enums::VertexFormat::Float32x4 => 16,
                    _ => 0,
                };
                (attr.offset + size as u64) as u32
            })
            .max()
            .unwrap_or(0)
    };

    // Create geometry resource
    let geometry_resource = GeometryResource {
        geometry_id: args.geometry_id,
        vertex_buffer,
        index_buffer,
        vertex_count: args.vertex_count,
        index_count: args.index_count,
        vertex_stride,
        index_format: args.index_format,
    };

    // Insert geometry resource
    render_state
        .resources
        .geometries
        .insert(args.geometry_id, geometry_resource);

    // Remove upload buffers after use
    engine.buffers.remove(&args.vertex_buffer_id);
    engine.buffers.remove(&args.index_buffer_id);

    CmdResultGeometryCreate {
        success: true,
        message: "Geometry resource created successfully".into(),
    }
}

// MARK: - Dispose Geometry

/// Arguments for disposing a geometry resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdGeometryDisposeArgs {
    pub geometry_id: GeometryId,
    pub window_id: u32,
}

impl Default for CmdGeometryDisposeArgs {
    fn default() -> Self {
        Self {
            geometry_id: 0,
            window_id: 0,
        }
    }
}

/// Result for geometry dispose command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultGeometryDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a geometry resource
pub fn engine_cmd_geometry_dispose(
    engine: &mut EngineState,
    args: &CmdGeometryDisposeArgs,
) -> CmdResultGeometryDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if geometry exists
    let in_use = render_state
        .components
        .models
        .values()
        .any(|m| m.geometry == args.geometry_id);

    if in_use {
        return CmdResultGeometryDispose {
            success: false,
            message: format!(
                "Geometry {} is still in use by one or more models",
                args.geometry_id
            ),
        };
    }

    // 🆕 Remove bindings
    render_state
        .binding_manager
        .remove_geometry_bindings(args.geometry_id);

    // Remove geometry resource
    match render_state.resources.geometries.remove(&args.geometry_id) {
        Some(_) => CmdResultGeometryDispose {
            success: true,
            message: "Geometry resource disposed successfully".into(),
        },
        None => CmdResultGeometryDispose {
            success: false,
            message: format!("Geometry with id {} not found", args.geometry_id),
        },
    }
}
