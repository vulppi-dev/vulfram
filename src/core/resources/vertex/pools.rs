use std::ops::Range;
use wgpu::{Buffer, BufferUsages, Device, Queue};

use super::types::{VertexAllocatorConfig, VertexStream};
use crate::core::resources::vertex::arena::{AllocHandle, ArenaAllocator};

// -----------------------------------------------------------------------------
// Stream pool (bytes allocator + fixed stride semantics)
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct StreamPool {
    stream: VertexStream,
    stride: u64,
    pub(super) arena: ArenaAllocator,
}

impl StreamPool {
    pub fn new(
        device: &Device,
        queue: &Queue,
        cfg: VertexAllocatorConfig,
        stream: VertexStream,
    ) -> Self {
        let stride = stream.stride_bytes();
        let usage = BufferUsages::VERTEX;
        let mut arena = ArenaAllocator::new(
            device,
            queue,
            cfg.min_pool_bytes.max(stride),
            usage,
            Some(match stream {
                VertexStream::Position => "Pool(Position)",
                VertexStream::Normal => "Pool(Normal)",
                VertexStream::Tangent => "Pool(Tangent)",
                VertexStream::Color0 => "Pool(Color0)",
                VertexStream::UV0 => "Pool(UV0)",
                VertexStream::UV1 => "Pool(UV1)",
                VertexStream::Joints => "Pool(Joints)",
                VertexStream::Weights => "Pool(Weights)",
            }),
        );
        arena.set_keep_frames(cfg.keep_frames);
        Self {
            stream,
            stride,
            arena,
        }
    }

    #[inline]
    pub fn buffer(&self) -> &Buffer {
        self.arena.buffer()
    }

    #[inline]
    pub fn slice_range(&self, h: AllocHandle) -> Range<u64> {
        self.arena.slice(h).range()
    }
}

// -----------------------------------------------------------------------------
// Index pool (u32 only)
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct IndexPool {
    pub(super) arena: ArenaAllocator,
}

impl IndexPool {
    pub fn new(device: &Device, queue: &Queue, cfg: VertexAllocatorConfig) -> Self {
        let mut arena = ArenaAllocator::new(
            device,
            queue,
            cfg.min_pool_bytes.max(4),
            BufferUsages::INDEX,
            Some("Pool(IndexU32)"),
        );
        arena.set_keep_frames(cfg.keep_frames);
        Self { arena }
    }

    #[inline]
    pub fn buffer(&self) -> &Buffer {
        self.arena.buffer()
    }

    #[inline]
    pub fn slice_range(&self, h: AllocHandle) -> Range<u64> {
        self.arena.slice(h).range()
    }
}
