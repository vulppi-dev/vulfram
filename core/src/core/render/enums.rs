use serde_repr::{Deserialize_repr, Serialize_repr};

// MARK: - Texture Enums

/// Texture format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum TextureFormat {
    // 8-bit formats
    R8Unorm = 0,
    R8Snorm = 1,
    R8Uint = 2,
    R8Sint = 3,
    // 16-bit formats
    R16Uint = 4,
    R16Sint = 5,
    R16Float = 6,
    Rg8Unorm = 7,
    Rg8Snorm = 8,
    Rg8Uint = 9,
    Rg8Sint = 10,
    // 32-bit formats
    R32Uint = 11,
    R32Sint = 12,
    R32Float = 13,
    Rg16Uint = 14,
    Rg16Sint = 15,
    Rg16Float = 16,
    Rgba8Unorm = 17,
    Rgba8UnormSrgb = 18,
    Rgba8Snorm = 19,
    Rgba8Uint = 20,
    Rgba8Sint = 21,
    Bgra8Unorm = 22,
    Bgra8UnormSrgb = 23,
    // Packed 32-bit formats
    Rgb10a2Unorm = 24,
    // 64-bit formats
    Rg32Uint = 25,
    Rg32Sint = 26,
    Rg32Float = 27,
    Rgba16Uint = 28,
    Rgba16Sint = 29,
    Rgba16Float = 30,
    // 128-bit formats
    Rgba32Uint = 31,
    Rgba32Sint = 32,
    Rgba32Float = 33,
    // Depth/stencil formats
    Depth32Float = 34,
    Depth24Plus = 35,
    Depth24PlusStencil8 = 36,
    Depth32FloatStencil8 = 37,
}

impl TextureFormat {
    pub fn to_wgpu(self) -> wgpu::TextureFormat {
        match self {
            Self::R8Unorm => wgpu::TextureFormat::R8Unorm,
            Self::R8Snorm => wgpu::TextureFormat::R8Snorm,
            Self::R8Uint => wgpu::TextureFormat::R8Uint,
            Self::R8Sint => wgpu::TextureFormat::R8Sint,
            Self::R16Uint => wgpu::TextureFormat::R16Uint,
            Self::R16Sint => wgpu::TextureFormat::R16Sint,
            Self::R16Float => wgpu::TextureFormat::R16Float,
            Self::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            Self::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
            Self::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
            Self::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
            Self::R32Uint => wgpu::TextureFormat::R32Uint,
            Self::R32Sint => wgpu::TextureFormat::R32Sint,
            Self::R32Float => wgpu::TextureFormat::R32Float,
            Self::Rg16Uint => wgpu::TextureFormat::Rg16Uint,
            Self::Rg16Sint => wgpu::TextureFormat::Rg16Sint,
            Self::Rg16Float => wgpu::TextureFormat::Rg16Float,
            Self::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            Self::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            Self::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            Self::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
            Self::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            Self::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            Self::Rgb10a2Unorm => wgpu::TextureFormat::Rgb10a2Unorm,
            Self::Rg32Uint => wgpu::TextureFormat::Rg32Uint,
            Self::Rg32Sint => wgpu::TextureFormat::Rg32Sint,
            Self::Rg32Float => wgpu::TextureFormat::Rg32Float,
            Self::Rgba16Uint => wgpu::TextureFormat::Rgba16Uint,
            Self::Rgba16Sint => wgpu::TextureFormat::Rgba16Sint,
            Self::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            Self::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
            Self::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
            Self::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
            Self::Depth32Float => wgpu::TextureFormat::Depth32Float,
            Self::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            Self::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
            Self::Depth32FloatStencil8 => wgpu::TextureFormat::Depth32FloatStencil8,
        }
    }
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum TextureUsage {
    CopySrc = 1,
    CopyDst = 2,
    TextureBinding = 4,
    StorageBinding = 8,
    RenderAttachment = 16,
}

impl TextureUsage {
    pub fn to_wgpu(self) -> wgpu::TextureUsages {
        match self {
            Self::CopySrc => wgpu::TextureUsages::COPY_SRC,
            Self::CopyDst => wgpu::TextureUsages::COPY_DST,
            Self::TextureBinding => wgpu::TextureUsages::TEXTURE_BINDING,
            Self::StorageBinding => wgpu::TextureUsages::STORAGE_BINDING,
            Self::RenderAttachment => wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }

    /// Convert array of usage flags to combined wgpu::TextureUsages
    pub fn combine(usages: &[Self]) -> wgpu::TextureUsages {
        usages.iter().fold(wgpu::TextureUsages::empty(), |acc, u| {
            acc | u.to_wgpu()
        })
    }
}

// MARK: - Vertex Format Enum

/// Vertex format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum VertexFormat {
    Uint8x2 = 0,
    Uint8x4 = 1,
    Sint8x2 = 2,
    Sint8x4 = 3,
    Unorm8x2 = 4,
    Unorm8x4 = 5,
    Snorm8x2 = 6,
    Snorm8x4 = 7,
    Uint16x2 = 8,
    Uint16x4 = 9,
    Sint16x2 = 10,
    Sint16x4 = 11,
    Unorm16x2 = 12,
    Unorm16x4 = 13,
    Snorm16x2 = 14,
    Snorm16x4 = 15,
    Float16x2 = 16,
    Float16x4 = 17,
    Float32 = 18,
    Float32x2 = 19,
    Float32x3 = 20,
    Float32x4 = 21,
    Uint32 = 22,
    Uint32x2 = 23,
    Uint32x3 = 24,
    Uint32x4 = 25,
    Sint32 = 26,
    Sint32x2 = 27,
    Sint32x3 = 28,
    Sint32x4 = 29,
    Float64 = 30,
    Float64x2 = 31,
    Float64x3 = 32,
    Float64x4 = 33,
}

impl VertexFormat {
    pub fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            Self::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            Self::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            Self::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            Self::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            Self::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            Self::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
            Self::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            Self::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            Self::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            Self::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            Self::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            Self::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            Self::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            Self::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            Self::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            Self::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            Self::Float16x2 => wgpu::VertexFormat::Float16x2,
            Self::Float16x4 => wgpu::VertexFormat::Float16x4,
            Self::Float32 => wgpu::VertexFormat::Float32,
            Self::Float32x2 => wgpu::VertexFormat::Float32x2,
            Self::Float32x3 => wgpu::VertexFormat::Float32x3,
            Self::Float32x4 => wgpu::VertexFormat::Float32x4,
            Self::Uint32 => wgpu::VertexFormat::Uint32,
            Self::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            Self::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            Self::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            Self::Sint32 => wgpu::VertexFormat::Sint32,
            Self::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            Self::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            Self::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            Self::Float64 => wgpu::VertexFormat::Float64,
            Self::Float64x2 => wgpu::VertexFormat::Float64x2,
            Self::Float64x3 => wgpu::VertexFormat::Float64x3,
            Self::Float64x4 => wgpu::VertexFormat::Float64x4,
        }
    }
}

/// Index format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum IndexFormat {
    Uint16 = 0,
    Uint32 = 1,
}

impl IndexFormat {
    pub fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::Uint16 => wgpu::IndexFormat::Uint16,
            Self::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

// MARK: - Material Enums

/// Blend factor enum for material blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    Src = 2,
    OneMinusSrc = 3,
    SrcAlpha = 4,
    OneMinusSrcAlpha = 5,
    Dst = 6,
    OneMinusDst = 7,
    DstAlpha = 8,
    OneMinusDstAlpha = 9,
    SrcAlphaSaturated = 10,
    Constant = 11,
    OneMinusConstant = 12,
}

impl BlendFactor {
    pub fn to_wgpu(self) -> wgpu::BlendFactor {
        match self {
            Self::Zero => wgpu::BlendFactor::Zero,
            Self::One => wgpu::BlendFactor::One,
            Self::Src => wgpu::BlendFactor::Src,
            Self::OneMinusSrc => wgpu::BlendFactor::OneMinusSrc,
            Self::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            Self::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            Self::Dst => wgpu::BlendFactor::Dst,
            Self::OneMinusDst => wgpu::BlendFactor::OneMinusDst,
            Self::DstAlpha => wgpu::BlendFactor::DstAlpha,
            Self::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            Self::SrcAlphaSaturated => wgpu::BlendFactor::SrcAlphaSaturated,
            Self::Constant => wgpu::BlendFactor::Constant,
            Self::OneMinusConstant => wgpu::BlendFactor::OneMinusConstant,
        }
    }
}

/// Blend operation enum for material blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum BlendOperation {
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

impl BlendOperation {
    pub fn to_wgpu(self) -> wgpu::BlendOperation {
        match self {
            Self::Add => wgpu::BlendOperation::Add,
            Self::Subtract => wgpu::BlendOperation::Subtract,
            Self::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
            Self::Min => wgpu::BlendOperation::Min,
            Self::Max => wgpu::BlendOperation::Max,
        }
    }
}

/// Compare function enum for depth/stencil testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum CompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl CompareFunction {
    pub fn to_wgpu(self) -> wgpu::CompareFunction {
        match self {
            Self::Never => wgpu::CompareFunction::Never,
            Self::Less => wgpu::CompareFunction::Less,
            Self::Equal => wgpu::CompareFunction::Equal,
            Self::LessEqual => wgpu::CompareFunction::LessEqual,
            Self::Greater => wgpu::CompareFunction::Greater,
            Self::NotEqual => wgpu::CompareFunction::NotEqual,
            Self::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            Self::Always => wgpu::CompareFunction::Always,
        }
    }
}

/// Stencil operation enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum StencilOperation {
    Keep = 0,
    Zero = 1,
    Replace = 2,
    Invert = 3,
    IncrementClamp = 4,
    DecrementClamp = 5,
    IncrementWrap = 6,
    DecrementWrap = 7,
}

impl StencilOperation {
    pub fn to_wgpu(self) -> wgpu::StencilOperation {
        match self {
            Self::Keep => wgpu::StencilOperation::Keep,
            Self::Zero => wgpu::StencilOperation::Zero,
            Self::Replace => wgpu::StencilOperation::Replace,
            Self::Invert => wgpu::StencilOperation::Invert,
            Self::IncrementClamp => wgpu::StencilOperation::IncrementClamp,
            Self::DecrementClamp => wgpu::StencilOperation::DecrementClamp,
            Self::IncrementWrap => wgpu::StencilOperation::IncrementWrap,
            Self::DecrementWrap => wgpu::StencilOperation::DecrementWrap,
        }
    }
}

/// Primitive topology enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum PrimitiveTopology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    TriangleList = 3,
    TriangleStrip = 4,
}

impl PrimitiveTopology {
    pub fn to_wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::PointList => wgpu::PrimitiveTopology::PointList,
            Self::LineList => wgpu::PrimitiveTopology::LineList,
            Self::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Self::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Self::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

/// Front face enum for triangle winding order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum FrontFace {
    Ccw = 0,
    Cw = 1,
}

impl FrontFace {
    pub fn to_wgpu(self) -> wgpu::FrontFace {
        match self {
            Self::Ccw => wgpu::FrontFace::Ccw,
            Self::Cw => wgpu::FrontFace::Cw,
        }
    }
}

/// Cull mode enum for face culling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum CullMode {
    Front = 0,
    Back = 1,
}

impl CullMode {
    pub fn to_wgpu(self) -> wgpu::Face {
        match self {
            Self::Front => wgpu::Face::Front,
            Self::Back => wgpu::Face::Back,
        }
    }
}

/// Polygon mode enum for rasterization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum PolygonMode {
    Fill = 0,
    Line = 1,
    Point = 2,
}

impl PolygonMode {
    pub fn to_wgpu(self) -> wgpu::PolygonMode {
        match self {
            Self::Fill => wgpu::PolygonMode::Fill,
            Self::Line => wgpu::PolygonMode::Line,
            Self::Point => wgpu::PolygonMode::Point,
        }
    }
}
