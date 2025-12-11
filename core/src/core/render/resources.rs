use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use wgpu;

use super::binding::ShaderUniformBuffers;
use super::buffers::{UniformBufferLayout, UniformField, UniformValue};
use super::enums::{IndexFormat, VertexFormat};

// MARK: - Logical IDs

pub type ShaderId = u32;
pub type GeometryId = u32;
pub type MaterialId = u32;
pub type TextureId = u32;
pub type SamplerId = u32;

// MARK: - Shader Bindings

/// Texture binding specification in shader
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextureBinding {
    pub group: u32,
    pub binding: u32,
    pub sample_type: TextureSampleType,
    pub view_dimension: TextureViewDimension,
}

/// Texture sample type
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum TextureSampleType {
    Float = 0,
    Depth,
    Sint,
    Uint,
}

/// Texture view dimension
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum TextureViewDimension {
    D1 = 0,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

/// Storage buffer binding specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageBufferBinding {
    pub group: u32,
    pub binding: u32,
    pub read_only: bool,
}

/// Uniform buffer binding specification (used in shader creation)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UniformBufferBinding {
    pub group: u32,
    pub binding: u32,
    pub fields: Vec<UniformField>,
}

/// Vertex attribute specification for shader
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VertexAttributeSpec {
    pub location: u32,
    pub semantic: VertexSemantic,
    pub format: VertexFormat,
}

/// Vertex semantic for attribute matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum VertexSemantic {
    Position = 0,
    Normal,
    Tangent,
    UV0,
    UV1,
    UV2,
    UV3,
    Color0,
    Color1,
    JointIndices,
    JointWeights,
}

// MARK: - Resources

/// ShaderResource holds a compiled shader module with metadata
pub struct ShaderResource {
    pub shader_id: ShaderId,
    pub module: wgpu::ShaderModule,

    // Interface metadata
    pub uniform_layouts: Vec<UniformBufferLayout>,

    // Vertex buffer layout (calculated from vertex_attributes during creation)
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,

    // Bind group layouts (one per group)
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    // Shader-owned uniform buffers with allocators
    pub uniform_buffers: ShaderUniformBuffers,
}

/// VertexAttributeDesc describes a single vertex attribute (command interface)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VertexAttributeDesc {
    pub format: VertexFormat,
    pub offset: u64,
    pub shader_location: u32,
}

impl Default for VertexAttributeDesc {
    fn default() -> Self {
        Self {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        }
    }
}

/// GeometryResource represents a mesh/geometry asset
pub struct GeometryResource {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub index_format: IndexFormat,
}

/// TextureParams describes texture creation parameters
#[derive(Debug, Clone)]
pub struct TextureParams {
    pub width: u32,
    pub height: u32,
    pub mip_level_count: u32,
}

/// TextureResource wraps a WGPU texture
pub struct TextureResource {
    pub texture: wgpu::Texture,
    pub params: TextureParams,
}

/// SamplerResource wraps a WGPU sampler
/// TODO: Integrate samplers with texture bindings in bind groups
pub struct SamplerResource {
    pub sampler: wgpu::Sampler,
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

/// MaterialResource represents everything needed to draw with a material
pub struct MaterialResource {
    pub pipeline_spec: PipelineSpec,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub textures: Vec<TextureId>,
    pub uniform_values: HashMap<String, UniformValue>,
}

// MARK: - Resource Manager

/// Resources holds all sharable resources indexed by logical IDs
pub struct Resources {
    pub shaders: HashMap<ShaderId, ShaderResource>,
    pub geometries: HashMap<GeometryId, GeometryResource>,
    pub materials: HashMap<MaterialId, MaterialResource>,
    pub textures: HashMap<TextureId, TextureResource>,
    pub samplers: HashMap<SamplerId, SamplerResource>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
            geometries: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            samplers: HashMap::new(),
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
        // Drop all samplers
        self.samplers.clear();
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
