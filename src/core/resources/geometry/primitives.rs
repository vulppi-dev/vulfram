use crate::core::resources::geometry::generators;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum PrimitiveShape {
    Cube,
    Plane,
    Sphere,
    Cylinder,
    Torus,
    Pyramid,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPrimitiveGeometryCreateArgs {
    pub window_id: u32,
    pub geometry_id: u32,
    pub shape: PrimitiveShape,
    // pub options: Option<...>, // To be defined later
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultPrimitiveGeometryCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_primitive_geometry_create(
    engine: &mut EngineState,
    args: &CmdPrimitiveGeometryCreateArgs,
) -> CmdResultPrimitiveGeometryCreate {
    // 1. Get window state
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultPrimitiveGeometryCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    // 2. Get vertex allocator
    let vertex_allocator = match window_state.render_state.vertex_allocation.as_mut() {
        Some(va) => va,
        None => {
            return CmdResultPrimitiveGeometryCreate {
                success: false,
                message: format!(
                    "Vertex allocator not initialized for window {}",
                    args.window_id
                ),
            };
        }
    };

    // 3. Generate data based on shape
    let geometry_data = match args.shape {
        PrimitiveShape::Cube => generators::generate_cube(),
        PrimitiveShape::Plane => generators::generate_plane(),
        PrimitiveShape::Pyramid => generators::generate_pyramid(),
        PrimitiveShape::Sphere => generators::generate_sphere(),
        PrimitiveShape::Cylinder => generators::generate_cylinder(),
        PrimitiveShape::Torus => generators::generate_torus(),
    };

    // 4. Create geometry using the vertex allocator
    match vertex_allocator.create_geometry(args.geometry_id, geometry_data) {
        Ok(_) => {
            window_state.is_dirty = true;
            CmdResultPrimitiveGeometryCreate {
                success: true,
                message: format!("Primitive geometry {:?} created successfully", args.shape),
            }
        }
        Err(e) => CmdResultPrimitiveGeometryCreate {
            success: false,
            message: format!("Failed to create primitive geometry: {:?}", e),
        },
    }
}
