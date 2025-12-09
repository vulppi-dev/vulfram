use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::core::render::enums::IndexFormat;
use crate::core::render::resources::{
    GeometryId, GeometryResource, VertexAttribute, VertexAttributeDesc,
};
use crate::core::state::EngineState;

// MARK: - Create Geometry

/// Arguments for creating a geometry resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdGeometryCreateArgs {
    pub geometry_id: GeometryId,
    pub window_id: u32,
    pub vertex_buffer_id: u64,
    pub index_buffer_id: Option<u64>,
    pub vertex_count: u32,
    pub index_count: Option<u32>,
    pub vertex_attributes: Vec<VertexAttributeDesc>,
    pub index_format: Option<IndexFormat>,
    pub label: Option<String>,
}

impl Default for CmdGeometryCreateArgs {
    fn default() -> Self {
        Self {
            geometry_id: 0,
            window_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: None,
            vertex_count: 0,
            index_count: None,
            vertex_attributes: Vec::new(),
            index_format: None,
            label: None,
        }
    }
}

/// Result for geometry creation command
#[derive(Debug, Serialize, Clone)]
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

    // Get or create render state
    if window_state.render_state.is_none() {
        window_state.render_state = Some(crate::core::render::RenderState::new());
    }

    let render_state = window_state.render_state.as_mut().unwrap();

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

    // Create index buffer if provided
    let index_buffer = if let Some(index_buffer_id) = args.index_buffer_id {
        let index_data = match engine.buffers.get(&index_buffer_id) {
            Some(buffer) => &buffer.data,
            None => {
                return CmdResultGeometryCreate {
                    success: false,
                    message: format!("Index upload buffer with id {} not found", index_buffer_id),
                };
            }
        };

        Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: args.label.as_deref(),
                contents: index_data,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
        )
    } else {
        None
    };

    // Convert vertex attributes
    let vertex_attributes: Vec<VertexAttribute> = args
        .vertex_attributes
        .iter()
        .map(|attr| VertexAttribute {
            format: attr.format.to_wgpu(),
            offset: attr.offset,
            shader_location: attr.shader_location,
        })
        .collect();

    // Convert index format
    let index_format = args.index_format;

    // Create geometry resource
    let geometry_resource = GeometryResource {
        geometry_id: args.geometry_id,
        vertex_buffer,
        index_buffer,
        vertex_count: args.vertex_count,
        index_count: args.index_count,
        vertex_attributes,
        index_format,
    };

    // Insert geometry resource
    render_state
        .resources
        .geometries
        .insert(args.geometry_id, geometry_resource);

    // Remove upload buffers after use
    engine.buffers.remove(&args.vertex_buffer_id);
    if let Some(index_buffer_id) = args.index_buffer_id {
        engine.buffers.remove(&index_buffer_id);
    }

    CmdResultGeometryCreate {
        success: true,
        message: "Geometry resource created successfully".into(),
    }
}

// MARK: - Dispose Geometry

/// Arguments for disposing a geometry resource
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Clone)]
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

    // Get render state
    let render_state = match &mut window_state.render_state {
        Some(rs) => rs,
        None => {
            return CmdResultGeometryDispose {
                success: false,
                message: "Window has no render state".into(),
            };
        }
    };

    // Check if geometry is in use by any models
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
