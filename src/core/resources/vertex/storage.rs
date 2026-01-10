use std::ops::Range;
use wgpu::Buffer;

use super::types::{IndexAlloc, IndexInfo, STREAM_COUNT};
use crate::core::resources::geometry::Aabb;
use crate::core::resources::vertex::arena::AllocHandle;

#[derive(Debug)]
pub enum GeometryStorage {
    Pooled {
        index: Option<IndexAlloc>,
        streams: [Option<AllocHandle>; STREAM_COUNT],
        vertex_count: u32,
    },
    Dedicated {
        buffer: Buffer,
        index: Option<(Range<u64>, IndexInfo)>,
        streams: [Option<Range<u64>>; STREAM_COUNT],
        vertex_count: u32,
    },
}

#[derive(Debug)]
pub struct GeometryRecord {
    pub alive: bool,
    pub storage: GeometryStorage,
    pub aabb: Aabb,
}
