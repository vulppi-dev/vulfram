use serde_repr::{Deserialize_repr, Serialize_repr};

// MARK: - Texture Enums

/// Texture format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum TextureFormat {
    // 8-bit formats
    R8Unorm = 0,
    R8Snorm,
    R8Uint,
    R8Sint,
    // 16-bit formats
    R16Uint,
    R16Sint,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,
    // 32-bit formats
    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    // Packed 32-bit formats
    Rgb10a2Unorm,
    // 64-bit formats
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Float,
    // 128-bit formats
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    // Depth/stencil formats
    Depth32Float,
    Depth24Plus,
    Depth24PlusStencil8,
    Depth32FloatStencil8,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
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
        usages
            .iter()
            .fold(wgpu::TextureUsages::empty(), |acc, u| acc | u.to_wgpu())
    }
}

// MARK: - Vertex Format Enum

/// Vertex format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum VertexFormat {
    Uint8x2 = 0,
    Uint8x4,
    Sint8x2,
    Sint8x4,
    Unorm8x2,
    Unorm8x4,
    Snorm8x2,
    Snorm8x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
    Unorm16x2,
    Unorm16x4,
    Snorm16x2,
    Snorm16x4,
    Float16x2,
    Float16x4,
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Float64,
    Float64x2,
    Float64x3,
    Float64x4,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum IndexFormat {
    Uint16 = 0,
    Uint32,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = 0,
    One,
    Src,
    OneMinusSrc,
    SrcAlpha,
    OneMinusSrcAlpha,
    Dst,
    OneMinusDst,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    Constant,
    OneMinusConstant,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum BlendOperation {
    Add = 0,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum CompareFunction {
    Never = 0,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum StencilOperation {
    Keep = 0,
    Zero,
    Replace,
    Invert,
    IncrementClamp,
    DecrementClamp,
    IncrementWrap,
    DecrementWrap,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum PrimitiveTopology {
    PointList = 0,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum FrontFace {
    Ccw = 0,
    Cw,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum CullMode {
    Front = 0,
    Back,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum PolygonMode {
    Fill = 0,
    Line,
    Point,
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

// MARK: - Sampler Enums

/// Address mode for texture sampling (wrap, clamp, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum AddressMode {
    ClampToEdge = 0,
    Repeat,
    MirrorRepeat,
    ClampToBorder,
}

impl AddressMode {
    pub fn to_wgpu(self) -> wgpu::AddressMode {
        match self {
            Self::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            Self::Repeat => wgpu::AddressMode::Repeat,
            Self::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
            Self::ClampToBorder => wgpu::AddressMode::ClampToBorder,
        }
    }
}

/// Filter mode for texture sampling (nearest, linear)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum FilterMode {
    Nearest = 0,
    Linear,
}

impl FilterMode {
    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            Self::Nearest => wgpu::FilterMode::Nearest,
            Self::Linear => wgpu::FilterMode::Linear,
        }
    }
}

/// Mipmap filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum MipmapFilterMode {
    Nearest = 0,
    Linear,
}

impl MipmapFilterMode {
    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            Self::Nearest => wgpu::FilterMode::Nearest,
            Self::Linear => wgpu::FilterMode::Linear,
        }
    }
}

/// Border color for ClampToBorder address mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum BorderColor {
    TransparentBlack = 0,
    OpaqueBlack,
    OpaqueWhite,
}

impl BorderColor {
    pub fn to_wgpu(self) -> Option<wgpu::SamplerBorderColor> {
        match self {
            Self::TransparentBlack => Some(wgpu::SamplerBorderColor::TransparentBlack),
            Self::OpaqueBlack => Some(wgpu::SamplerBorderColor::OpaqueBlack),
            Self::OpaqueWhite => Some(wgpu::SamplerBorderColor::OpaqueWhite),
        }
    }
}
