use serde_repr::{Deserialize_repr, Serialize_repr};

use super::arena::AllocHandle;

// -----------------------------------------------------------------------------
// Fixed vertex streams (8 total) + fixed shader locations
// -----------------------------------------------------------------------------
pub const STREAM_COUNT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexStream {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Color0 = 3,
    UV0 = 4,
    UV1 = 5,
    Joints = 6,
    Weights = 7,
}

impl VertexStream {
    #[inline]
    pub fn slot(self) -> u32 {
        self as u32
    }

    #[inline]
    pub fn stride_bytes(self) -> u64 {
        match self {
            VertexStream::Position => 12, // f32x3
            VertexStream::Normal => 12,   // f32x3
            VertexStream::Tangent => 16,  // f32x4
            VertexStream::Color0 => 16,   // f32x4
            VertexStream::UV0 => 8,       // f32x2
            VertexStream::UV1 => 8,       // f32x2
            VertexStream::Joints => 8,    // u16x4
            VertexStream::Weights => 16,  // f32x4
        }
    }
}

#[inline]
pub fn all_streams() -> [VertexStream; STREAM_COUNT] {
    [
        VertexStream::Position,
        VertexStream::Normal,
        VertexStream::Tangent,
        VertexStream::Color0,
        VertexStream::UV0,
        VertexStream::UV1,
        VertexStream::Joints,
        VertexStream::Weights,
    ]
}

// -----------------------------------------------------------------------------
// Configuration
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct VertexAllocatorConfig {
    pub min_pool_bytes: u64,            // >= 2MB
    pub dedicated_threshold_bytes: u64, // > 16MB => Dedicated
    pub keep_frames: u64,               // deferred drop window for arena resizes/compactions
}

impl Default for VertexAllocatorConfig {
    fn default() -> Self {
        Self {
            min_pool_bytes: 2 * 1024 * 1024,
            dedicated_threshold_bytes: 16 * 1024 * 1024,
            keep_frames: 3,
        }
    }
}

// -----------------------------------------------------------------------------
// Geometry Primitive Type (input format)
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum GeometryPrimitiveType {
    Index = 0,
    Position,
    Normal,
    Tangent,
    Color,
    UV,
    SkinJoints,
    SkinWeights,
}

// -----------------------------------------------------------------------------
// Index info (u32 only)
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct IndexInfo {
    pub count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexAlloc {
    pub handle: AllocHandle,
    pub info: IndexInfo,
}

// -----------------------------------------------------------------------------
// Errors
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum VertexAllocError {
    MissingPosition,
    DuplicateIndex,
    DuplicateStream(GeometryPrimitiveType),
    TooManyUVSets,
    InvalidStride {
        stream: crate::core::resources::vertex::VertexStream,
        byte_len: usize,
        stride: u64,
    },
    InvalidIndexBytes {
        byte_len: usize,
        reason: &'static str,
    },
    PositionCountMismatch {
        expected: u32,
        got: u32,
        stream: crate::core::resources::vertex::VertexStream,
    },
    GeometryNotFound,
}

impl std::fmt::Display for VertexAllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use VertexAllocError::*;
        match self {
            MissingPosition => write!(f, "geometry missing mandatory Position stream"),
            DuplicateIndex => write!(f, "geometry has multiple Index entries"),
            DuplicateStream(t) => write!(f, "geometry has duplicate stream entry: {:?}", t),
            TooManyUVSets => write!(f, "geometry provided more than 2 UV streams (UV0/UV1 only)"),
            InvalidStride {
                stream,
                byte_len,
                stride,
            } => write!(
                f,
                "stream {:?} byte length {} not multiple of stride {}",
                stream, byte_len, stride
            ),
            InvalidIndexBytes { byte_len, reason } => write!(
                f,
                "invalid index buffer byte length {}: {}",
                byte_len, reason
            ),
            PositionCountMismatch {
                expected,
                got,
                stream,
            } => write!(
                f,
                "stream {:?} vertex count {} mismatches Position count {}",
                stream, got, expected
            ),
            GeometryNotFound => write!(f, "geometry id not found"),
        }
    }
}

impl std::error::Error for VertexAllocError {}
