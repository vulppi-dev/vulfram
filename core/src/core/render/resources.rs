use std::collections::HashMap;
use wgpu;

// MARK: - Logical IDs

pub type ShaderId = u32;
pub type GeometryId = u32;
pub type MaterialId = u32;
pub type TextureId = u32;

// MARK: - Resources

/// ShaderResource holds a compiled shader module
pub struct ShaderResource {
    pub shader_id: ShaderId,
    pub module: wgpu::ShaderModule,
}

/// GeometryBuffers describes the buffer IDs and layout for geometry data
#[derive(Debug, Clone)]
pub struct GeometryBuffers {
    pub vertex_buffer_id: u32,
    pub index_buffer_id: Option<u32>,
    pub vertex_attributes: Vec<VertexAttributeDesc>,
    pub index_format: Option<wgpu::IndexFormat>,
}

/// VertexAttributeDesc describes a single vertex attribute
#[derive(Debug, Clone)]
pub struct VertexAttributeDesc {
    pub format: wgpu::VertexFormat,
    pub offset: u64,
    pub shader_location: u32,
}

/// GeometryResource represents a mesh/geometry asset
pub struct GeometryResource {
    pub geometry_id: GeometryId,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count: Option<u32>,
    pub vertex_attributes: Vec<VertexAttributeDesc>,
    pub index_format: Option<wgpu::IndexFormat>,
}

/// TextureParams describes texture creation parameters
#[derive(Debug, Clone)]
pub struct TextureParams {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub mip_level_count: u32,
}

/// TextureResource wraps a WGPU texture and view
pub struct TextureResource {
    pub texture_id: TextureId,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub params: TextureParams,
}

/// PipelineSpec is a logical description of a render pipeline
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PipelineSpec {
    pub shader_id: ShaderId,
    pub blend: Option<wgpu::BlendState>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
}

/// MaterialParams describes material creation parameters
#[derive(Debug, Clone)]
pub struct MaterialParams {
    pub shader_id: ShaderId,
    pub textures: Vec<TextureId>,
    pub blend: Option<wgpu::BlendState>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub primitive: wgpu::PrimitiveState,
}

/// MaterialResource represents everything needed to draw with a material
pub struct MaterialResource {
    pub material_id: MaterialId,
    pub pipeline_spec: PipelineSpec,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub uniform_offset: u32,
    pub storage_offset: Option<u32>,
    pub textures: Vec<TextureId>,
}

// MARK: - Resource Manager

/// Resources holds all sharable resources indexed by logical IDs
pub struct Resources {
    pub shaders: HashMap<ShaderId, ShaderResource>,
    pub geometries: HashMap<GeometryId, GeometryResource>,
    pub materials: HashMap<MaterialId, MaterialResource>,
    pub textures: HashMap<TextureId, TextureResource>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            geometries: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    /// Explicitly drop all resources and their GPU handles
    /// Use this when closing a window or disposing the engine
    pub fn drop_all(&mut self) {
        // Drop all shaders (includes ShaderModule)
        self.shaders.clear();
        // Drop all geometries (includes vertex/index buffers)
        self.geometries.clear();
        // Drop all materials (includes pipelines)
        self.materials.clear();
        // Drop all textures (includes texture and view)
        self.textures.clear();
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
