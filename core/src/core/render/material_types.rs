use serde::{Deserialize, Serialize};

use super::enums::{
    BlendFactor, BlendOperation, CompareFunction, CullMode, FrontFace, IndexFormat, PolygonMode,
    PrimitiveTopology, StencilOperation, TextureFormat,
};

// MARK: - Blend State

/// Blend state descriptor for command deserialization
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct BlendStateDesc {
    pub color: BlendComponentDesc,
    pub alpha: BlendComponentDesc,
}

impl Default for BlendStateDesc {
    fn default() -> Self {
        Self {
            color: BlendComponentDesc::default(),
            alpha: BlendComponentDesc::default(),
        }
    }
}

/// Blend component descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct BlendComponentDesc {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

impl Default for BlendComponentDesc {
    fn default() -> Self {
        Self {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        }
    }
}

// MARK: - Depth Stencil State

/// Depth stencil state descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct DepthStencilStateDesc {
    pub format: TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil: StencilStateDesc,
    pub bias: DepthBiasStateDesc,
}

impl Default for DepthStencilStateDesc {
    fn default() -> Self {
        Self {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilStateDesc::default(),
            bias: DepthBiasStateDesc::default(),
        }
    }
}

/// Stencil state descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct StencilStateDesc {
    pub front: StencilFaceStateDesc,
    pub back: StencilFaceStateDesc,
    pub read_mask: u32,
    pub write_mask: u32,
}

impl Default for StencilStateDesc {
    fn default() -> Self {
        Self {
            front: StencilFaceStateDesc::default(),
            back: StencilFaceStateDesc::default(),
            read_mask: 0xFF,
            write_mask: 0xFF,
        }
    }
}

/// Stencil face state descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct StencilFaceStateDesc {
    pub compare: CompareFunction,
    pub fail_op: StencilOperation,
    pub depth_fail_op: StencilOperation,
    pub pass_op: StencilOperation,
}

impl Default for StencilFaceStateDesc {
    fn default() -> Self {
        Self {
            compare: CompareFunction::Always,
            fail_op: StencilOperation::Keep,
            depth_fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}

/// Depth bias state descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct DepthBiasStateDesc {
    pub constant: i32,
    pub slope_scale: f32,
    pub clamp: f32,
}

impl Default for DepthBiasStateDesc {
    fn default() -> Self {
        Self {
            constant: 0,
            slope_scale: 0.0,
            clamp: 0.0,
        }
    }
}

// MARK: - Primitive State

/// Primitive state descriptor
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct PrimitiveStateDesc {
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub front_face: FrontFace,
    pub cull_mode: Option<CullMode>,
    pub unclipped_depth: bool,
    pub polygon_mode: PolygonMode,
    pub conservative: bool,
}

impl Default for PrimitiveStateDesc {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(CullMode::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        }
    }
}
