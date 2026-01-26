use crate::core::resources::geometry::generators;
use crate::core::state::EngineState;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
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
pub struct CubeOptions {
    pub size: Vec3,
    pub subdivisions: u32,
}
impl Default for CubeOptions {
    fn default() -> Self {
        Self {
            size: Vec3::ONE,
            subdivisions: 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaneOptions {
    pub size: Vec3,
    pub subdivisions: u32,
}
impl Default for PlaneOptions {
    fn default() -> Self {
        Self {
            size: Vec3::new(1.0, 1.0, 1.0),
            subdivisions: 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SphereOptions {
    pub radius: f32,
    pub sectors: u32,
    pub stacks: u32,
}
impl Default for SphereOptions {
    fn default() -> Self {
        Self {
            radius: 0.5,
            sectors: 32,
            stacks: 16,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CylinderOptions {
    pub radius: f32,
    pub height: f32,
    pub sectors: u32,
}
impl Default for CylinderOptions {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            sectors: 32,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TorusOptions {
    pub major_radius: f32,
    pub minor_radius: f32,
    pub major_segments: u32,
    pub minor_segments: u32,
}
impl Default for TorusOptions {
    fn default() -> Self {
        Self {
            major_radius: 0.4,
            minor_radius: 0.1,
            major_segments: 32,
            minor_segments: 16,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PyramidOptions {
    pub size: Vec3,
    pub subdivisions: u32,
}
impl Default for PyramidOptions {
    fn default() -> Self {
        Self {
            size: Vec3::ONE,
            subdivisions: 1,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum PrimitiveOptions {
    Cube(CubeOptions),
    Plane(PlaneOptions),
    Sphere(SphereOptions),
    Cylinder(CylinderOptions),
    Torus(TorusOptions),
    Pyramid(PyramidOptions),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPrimitiveGeometryCreateArgs {
    pub window_id: u32,
    pub geometry_id: u32,
    pub label: Option<String>,
    pub shape: PrimitiveShape,
    #[serde(default)]
    pub options: Option<PrimitiveOptions>,
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
    let options = match (args.shape, &args.options) {
        (PrimitiveShape::Cube, Some(PrimitiveOptions::Cube(opts))) => {
            PrimitiveOptions::Cube(opts.clone())
        }
        (PrimitiveShape::Cube, None) => PrimitiveOptions::Cube(CubeOptions::default()),
        (PrimitiveShape::Plane, Some(PrimitiveOptions::Plane(opts))) => {
            PrimitiveOptions::Plane(opts.clone())
        }
        (PrimitiveShape::Plane, None) => PrimitiveOptions::Plane(PlaneOptions::default()),
        (PrimitiveShape::Sphere, Some(PrimitiveOptions::Sphere(opts))) => {
            PrimitiveOptions::Sphere(opts.clone())
        }
        (PrimitiveShape::Sphere, None) => PrimitiveOptions::Sphere(SphereOptions::default()),
        (PrimitiveShape::Cylinder, Some(PrimitiveOptions::Cylinder(opts))) => {
            PrimitiveOptions::Cylinder(opts.clone())
        }
        (PrimitiveShape::Cylinder, None) => PrimitiveOptions::Cylinder(CylinderOptions::default()),
        (PrimitiveShape::Torus, Some(PrimitiveOptions::Torus(opts))) => {
            PrimitiveOptions::Torus(opts.clone())
        }
        (PrimitiveShape::Torus, None) => PrimitiveOptions::Torus(TorusOptions::default()),
        (PrimitiveShape::Pyramid, Some(PrimitiveOptions::Pyramid(opts))) => {
            PrimitiveOptions::Pyramid(opts.clone())
        }
        (PrimitiveShape::Pyramid, None) => PrimitiveOptions::Pyramid(PyramidOptions::default()),
        (shape, Some(_)) => {
            return CmdResultPrimitiveGeometryCreate {
                success: false,
                message: format!("Options type mismatch for {:?}", shape),
            };
        }
    };

    if let Err(message) = validate_options(&options) {
        return CmdResultPrimitiveGeometryCreate {
            success: false,
            message,
        };
    }

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
    let vertex_allocator = match window_state.render_state.vertex.as_mut() {
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
    let geometry_data = match options {
        PrimitiveOptions::Cube(opts) => generators::generate_cube(&opts),
        PrimitiveOptions::Plane(opts) => generators::generate_plane(&opts),
        PrimitiveOptions::Sphere(opts) => generators::generate_sphere(&opts),
        PrimitiveOptions::Cylinder(opts) => generators::generate_cylinder(&opts),
        PrimitiveOptions::Torus(opts) => generators::generate_torus(&opts),
        PrimitiveOptions::Pyramid(opts) => generators::generate_pyramid(&opts),
    };

    // 4. Create geometry using the vertex allocator
    match vertex_allocator.create_geometry(args.geometry_id, args.label.clone(), geometry_data) {
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

fn validate_options(options: &PrimitiveOptions) -> Result<(), String> {
    match options {
        PrimitiveOptions::Cube(opts) => {
            if opts.subdivisions == 0 {
                return Err("Cube subdivisions must be >= 1".to_string());
            }
        }
        PrimitiveOptions::Plane(opts) => {
            if opts.subdivisions == 0 {
                return Err("Plane subdivisions must be >= 1".to_string());
            }
        }
        PrimitiveOptions::Sphere(opts) => {
            if opts.sectors < 3 {
                return Err("Sphere sectors must be >= 3".to_string());
            }
            if opts.stacks < 2 {
                return Err("Sphere stacks must be >= 2".to_string());
            }
        }
        PrimitiveOptions::Cylinder(opts) => {
            if opts.sectors < 3 {
                return Err("Cylinder sectors must be >= 3".to_string());
            }
        }
        PrimitiveOptions::Torus(opts) => {
            if opts.major_segments < 3 {
                return Err("Torus major_segments must be >= 3".to_string());
            }
            if opts.minor_segments < 3 {
                return Err("Torus minor_segments must be >= 3".to_string());
            }
        }
        PrimitiveOptions::Pyramid(opts) => {
            if opts.subdivisions == 0 {
                return Err("Pyramid subdivisions must be >= 1".to_string());
            }
        }
    }

    Ok(())
}
