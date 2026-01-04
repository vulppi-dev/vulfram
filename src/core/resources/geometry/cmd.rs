use serde::{Deserialize, Serialize};

use crate::core::resources::vertex::GeometryPrimitiveType;
use crate::core::state::EngineState;

// -----------------------------------------------------------------------------
// GeometryPrimitiveEntry
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeometryPrimitiveEntry {
    pub primitive_type: GeometryPrimitiveType,
    pub buffer_id: u64,
}

// -----------------------------------------------------------------------------
// Create
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryCreateArgs {
    pub window_id: u32,
    pub geometry_id: u32,
    pub entries: Vec<GeometryPrimitiveEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_geometry_create(
    engine: &mut EngineState,
    args: &CmdGeometryCreateArgs,
) -> CmdResultGeometryCreate {
    // 1. Validar window
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    // 2. Validar que temos vertex_allocator
    let vertex_allocator = match window_state.render_state.vertex.as_mut() {
        Some(va) => va,
        None => {
            return CmdResultGeometryCreate {
                success: false,
                message: format!(
                    "Vertex allocator not initialized for window {}",
                    args.window_id
                ),
            };
        }
    };

    // 3. Validar buffers existem
    for entry in &args.entries {
        if !engine.buffers.uploads.contains_key(&entry.buffer_id) {
            return CmdResultGeometryCreate {
                success: false,
                message: format!("Buffer {} not found", entry.buffer_id),
            };
        }
    }

    // 4. Validar tipos primitivos
    let has_position = args
        .entries
        .iter()
        .any(|e| matches!(e.primitive_type, GeometryPrimitiveType::Position));

    if !has_position {
        return CmdResultGeometryCreate {
            success: false,
            message: "Position primitive is required".into(),
        };
    }

    let uv_count = args
        .entries
        .iter()
        .filter(|e| matches!(e.primitive_type, GeometryPrimitiveType::UV))
        .count();

    if uv_count > 2 {
        return CmdResultGeometryCreate {
            success: false,
            message: format!("Too many UV sets (max 2, got {})", uv_count),
        };
    }

    // Verificar duplicatas (exceto UV)
    let mut seen_types = std::collections::HashSet::new();
    for entry in &args.entries {
        if !matches!(entry.primitive_type, GeometryPrimitiveType::UV) {
            if !seen_types.insert(entry.primitive_type as u32) {
                return CmdResultGeometryCreate {
                    success: false,
                    message: format!("Duplicate primitive type: {:?}", entry.primitive_type),
                };
            }
        }
    }

    // 5. Montar dados
    let mut geometry_data = Vec::new();
    for entry in &args.entries {
        let buffer = match engine.buffers.uploads.get(&entry.buffer_id) {
            Some(buffer) => buffer,
            None => {
                return CmdResultGeometryCreate {
                    success: false,
                    message: format!("Buffer {} not found", entry.buffer_id),
                };
            }
        };
        geometry_data.push((entry.primitive_type, buffer.data.clone()));
    }

    // 6. Criar geometria
    match vertex_allocator.create_geometry(args.geometry_id, geometry_data) {
        Ok(_) => {
            // 7. Limpar buffers apenas em caso de sucesso
            for entry in &args.entries {
                engine.buffers.uploads.remove(&entry.buffer_id);
            }

            window_state.is_dirty = true;

            CmdResultGeometryCreate {
                success: true,
                message: "Geometry created successfully".into(),
            }
        }
        Err(e) => {
            // Buffers NÃO são removidos para permitir retry
            CmdResultGeometryCreate {
                success: false,
                message: format!("Vertex allocator error: {:?}", e),
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Update
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGeometryUpdateArgs {
    pub window_id: u32,
    pub geometry_id: u32,
    pub entries: Vec<GeometryPrimitiveEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryUpdate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_geometry_update(
    engine: &mut EngineState,
    args: &CmdGeometryUpdateArgs,
) -> CmdResultGeometryUpdate {
    // Update é idêntico ao create - o VertexAllocatorSystem
    // já trata o replace automático

    // 1. Validar window
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    // 2. Validar que temos vertex_allocator
    let vertex_allocator = match window_state.render_state.vertex.as_mut() {
        Some(va) => va,
        None => {
            return CmdResultGeometryUpdate {
                success: false,
                message: format!(
                    "Vertex allocator not initialized for window {}",
                    args.window_id
                ),
            };
        }
    };

    // 3. Validar buffers existem
    for entry in &args.entries {
        if !engine.buffers.uploads.contains_key(&entry.buffer_id) {
            return CmdResultGeometryUpdate {
                success: false,
                message: format!("Buffer {} not found", entry.buffer_id),
            };
        }
    }

    // 4. Validar tipos primitivos
    let has_position = args
        .entries
        .iter()
        .any(|e| matches!(e.primitive_type, GeometryPrimitiveType::Position));

    if !has_position {
        return CmdResultGeometryUpdate {
            success: false,
            message: "Position primitive is required".into(),
        };
    }

    let uv_count = args
        .entries
        .iter()
        .filter(|e| matches!(e.primitive_type, GeometryPrimitiveType::UV))
        .count();

    if uv_count > 2 {
        return CmdResultGeometryUpdate {
            success: false,
            message: format!("Too many UV sets (max 2, got {})", uv_count),
        };
    }

    // Verificar duplicatas (exceto UV)
    let mut seen_types = std::collections::HashSet::new();
    for entry in &args.entries {
        if !matches!(entry.primitive_type, GeometryPrimitiveType::UV) {
            if !seen_types.insert(entry.primitive_type as u32) {
                return CmdResultGeometryUpdate {
                    success: false,
                    message: format!("Duplicate primitive type: {:?}", entry.primitive_type),
                };
            }
        }
    }

    // 5. Montar dados
    let mut geometry_data = Vec::new();
    for entry in &args.entries {
        let buffer = match engine.buffers.uploads.get(&entry.buffer_id) {
            Some(buffer) => buffer,
            None => {
                return CmdResultGeometryUpdate {
                    success: false,
                    message: format!("Buffer {} not found", entry.buffer_id),
                };
            }
        };
        geometry_data.push((entry.primitive_type, buffer.data.clone()));
    }

    // 6. Atualizar geometria (create_geometry já trata replace)
    match vertex_allocator.create_geometry(args.geometry_id, geometry_data) {
        Ok(_) => {
            // 7. Limpar buffers apenas em caso de sucesso
            for entry in &args.entries {
                engine.buffers.uploads.remove(&entry.buffer_id);
            }

            window_state.is_dirty = true;

            CmdResultGeometryUpdate {
                success: true,
                message: "Geometry updated successfully".into(),
            }
        }
        Err(e) => {
            // Buffers NÃO são removidos para permitir retry
            CmdResultGeometryUpdate {
                success: false,
                message: format!("Vertex allocator error: {:?}", e),
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Dispose
// -----------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdGeometryDisposeArgs {
    pub window_id: u32,
    pub geometry_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultGeometryDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_geometry_dispose(
    engine: &mut EngineState,
    args: &CmdGeometryDisposeArgs,
) -> CmdResultGeometryDispose {
    // 1. Validar window
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultGeometryDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    // 2. Validar que temos vertex_allocator
    let vertex_allocator = match window_state.render_state.vertex.as_mut() {
        Some(va) => va,
        None => {
            return CmdResultGeometryDispose {
                success: false,
                message: format!(
                    "Vertex allocator not initialized for window {}",
                    args.window_id
                ),
            };
        }
    };

    // 3. Destruir geometria
    match vertex_allocator.destroy_geometry(args.geometry_id) {
        Ok(_) => {
            window_state.is_dirty = true;

            CmdResultGeometryDispose {
                success: true,
                message: "Geometry disposed successfully".into(),
            }
        }
        Err(e) => CmdResultGeometryDispose {
            success: false,
            message: format!("Failed to dispose geometry: {:?}", e),
        },
    }
}
