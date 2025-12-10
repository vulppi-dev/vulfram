use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextureSampleType {
    Float,
    Depth,
    Sint,
    Uint,
}

/// Texture view dimension
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextureViewDimension {
    D1,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VertexSemantic {
    Position,
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
    #[allow(dead_code)]
    pub texture_bindings: Vec<TextureBinding>,
    #[allow(dead_code)]
    pub storage_buffers: Vec<StorageBufferBinding>,
    #[allow(dead_code)]
    pub vertex_attributes: Vec<VertexAttributeSpec>,

    // 🆕 NEW: Vertex buffer layout (calculated from vertex_attributes)
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,

    // 🆕 NEW: Bind group layouts (one per group)
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    // 🆕 NEW: Shader-owned uniform buffers with allocators
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
    #[allow(dead_code)]
    pub geometry_id: GeometryId,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    pub vertex_count: u32,
    pub index_count: u32,
    /// 🆕 Vertex stride in bytes (replaces vertex_attributes)
    /// Vertex layout comes from the shader, not the geometry
    #[allow(dead_code)]
    pub vertex_stride: u32,
    pub index_format: IndexFormat,
}

/// TextureParams describes texture creation parameters
#[derive(Debug, Clone)]
pub struct TextureParams {
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub format: wgpu::TextureFormat,
    #[allow(dead_code)]
    pub usage: wgpu::TextureUsages,
    pub mip_level_count: u32,
}

/// TextureResource wraps a WGPU texture and view
pub struct TextureResource {
    #[allow(dead_code)]
    pub texture_id: TextureId,
    pub texture: wgpu::Texture,
    #[allow(dead_code)]
    pub view: wgpu::TextureView,
    pub params: TextureParams,
}

/// SamplerParams describes sampler creation parameters
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SamplerParams {
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
    pub address_mode_w: wgpu::AddressMode,
    pub mag_filter: wgpu::FilterMode,
    pub min_filter: wgpu::FilterMode,
    pub mipmap_filter: wgpu::FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<wgpu::CompareFunction>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<wgpu::SamplerBorderColor>,
}

/// SamplerResource wraps a WGPU sampler
#[allow(dead_code)]
pub struct SamplerResource {
    pub sampler_id: SamplerId,
    pub sampler: wgpu::Sampler,
    pub params: SamplerParams,
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
    #[allow(dead_code)]
    pub material_id: MaterialId,
    pub pipeline_spec: PipelineSpec,
    /// Render pipeline (created lazily on first draw)
    /// Pipeline is built from shader + pipeline_spec
    pub pipeline: Option<wgpu::RenderPipeline>,
    /// Textures used by this material
    pub textures: Vec<TextureId>,
    /// Custom uniform values for material-specific parameters
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
