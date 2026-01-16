use std::collections::HashMap;
use wgpu::{Device, Queue};

mod alloc;
mod arena;
mod bind;
mod bind_cache;
mod defaults;
mod storage;
mod types;

use arena::*;
use bind_cache::*;
use defaults::*;
use storage::*;
use types::*;

use crate::core::resources::geometry::Aabb;

pub use types::{
    GeometryPrimitiveType, IndexInfo, STREAM_COUNT, VertexAllocError, VertexAllocatorConfig,
    VertexStream,
};

// -----------------------------------------------------------------------------
// VertexAllocatorSystem
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct VertexAllocatorSystem {
    cfg: VertexAllocatorConfig,
    device: wgpu::Device,
    queue: wgpu::Queue,

    index_u32: ArenaAllocator,
    streams: [ArenaAllocator; STREAM_COUNT],

    default_normal: DefaultStreamBuffer,
    default_tangent: DefaultStreamBuffer,
    default_color0: DefaultStreamBuffer,
    default_uv0: DefaultStreamBuffer,
    default_uv1: DefaultStreamBuffer,
    default_joints: DefaultStreamBuffer,
    default_weights: DefaultStreamBuffer,

    records: HashMap<u32, GeometryRecord>,

    bind_cache: BindCache,
}

impl VertexAllocatorSystem {
    pub fn records(&self) -> &HashMap<u32, GeometryRecord> {
        &self.records
    }

    pub fn records_mut(&mut self) -> &mut HashMap<u32, GeometryRecord> {
        &mut self.records
    }
}

impl VertexAllocatorSystem {
    pub fn new(device: &Device, queue: &Queue, cfg: VertexAllocatorConfig) -> Self {
        let mut index_u32 = ArenaAllocator::new(
            device,
            queue,
            cfg.min_pool_bytes.max(4),
            wgpu::BufferUsages::INDEX,
            Some("Pool(IndexU32)"),
        );
        index_u32.set_keep_frames(cfg.keep_frames);

        let mut streams = [
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes
                    .max(VertexStream::Position.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Position)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::Normal.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Normal)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::Tangent.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Tangent)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::Color0.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Color0)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::UV0.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(UV0)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::UV1.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(UV1)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::Joints.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Joints)"),
            ),
            ArenaAllocator::new(
                device,
                queue,
                cfg.min_pool_bytes.max(VertexStream::Weights.stride_bytes()),
                wgpu::BufferUsages::VERTEX,
                Some("Pool(Weights)"),
            ),
        ];

        for arena in &mut streams {
            arena.set_keep_frames(cfg.keep_frames);
        }

        let mut sys = Self {
            cfg,
            device: device.clone(),
            queue: queue.clone(),
            index_u32,
            streams,
            default_normal: DefaultStreamBuffer::new(device, cfg, VertexStream::Normal),
            default_tangent: DefaultStreamBuffer::new(device, cfg, VertexStream::Tangent),
            default_color0: DefaultStreamBuffer::new(device, cfg, VertexStream::Color0),
            default_uv0: DefaultStreamBuffer::new(device, cfg, VertexStream::UV0),
            default_uv1: DefaultStreamBuffer::new(device, cfg, VertexStream::UV1),
            default_joints: DefaultStreamBuffer::new(device, cfg, VertexStream::Joints),
            default_weights: DefaultStreamBuffer::new(device, cfg, VertexStream::Weights),
            records: HashMap::new(),
            bind_cache: BindCache::default(),
        };

        sys.initialize_default_buffers();
        sys
    }

    fn initialize_default_buffers(&mut self) {
        let caps = [
            self.default_normal.capacity_vertices,
            self.default_tangent.capacity_vertices,
            self.default_color0.capacity_vertices,
            self.default_uv0.capacity_vertices,
            self.default_uv1.capacity_vertices,
            self.default_joints.capacity_vertices,
            self.default_weights.capacity_vertices,
        ];

        let fill =
            build_repeated_default_bytes(VertexStream::Normal, self.default_normal.stride, caps[0]);
        self.queue
            .write_buffer(&self.default_normal.buffer, 0, &fill);

        let fill = build_repeated_default_bytes(
            VertexStream::Tangent,
            self.default_tangent.stride,
            caps[1],
        );
        self.queue
            .write_buffer(&self.default_tangent.buffer, 0, &fill);

        let fill =
            build_repeated_default_bytes(VertexStream::Color0, self.default_color0.stride, caps[2]);
        self.queue
            .write_buffer(&self.default_color0.buffer, 0, &fill);

        let fill =
            build_repeated_default_bytes(VertexStream::UV0, self.default_uv0.stride, caps[3]);
        self.queue.write_buffer(&self.default_uv0.buffer, 0, &fill);

        let fill =
            build_repeated_default_bytes(VertexStream::UV1, self.default_uv1.stride, caps[4]);
        self.queue.write_buffer(&self.default_uv1.buffer, 0, &fill);

        let fill =
            build_repeated_default_bytes(VertexStream::Joints, self.default_joints.stride, caps[5]);
        self.queue
            .write_buffer(&self.default_joints.buffer, 0, &fill);

        let fill = build_repeated_default_bytes(
            VertexStream::Weights,
            self.default_weights.stride,
            caps[6],
        );
        self.queue
            .write_buffer(&self.default_weights.buffer, 0, &fill);
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        self.index_u32.begin_frame(frame_index);
        for p in &mut self.streams {
            p.begin_frame(frame_index);
        }
        self.bind_cache.reset();
    }

    pub fn begin_pass(&mut self) {
        // Render passes do not carry vertex bindings; reset cache to force rebinds.
        self.bind_cache.reset();
    }

    pub fn index_info(&self, id: u32) -> Result<Option<IndexInfo>, VertexAllocError> {
        let rec = self
            .records
            .get(&id)
            .ok_or(VertexAllocError::GeometryNotFound)?;
        if !rec.alive {
            return Err(VertexAllocError::GeometryNotFound);
        }
        Ok(match &rec.storage {
            GeometryStorage::Pooled { index, .. } => index.as_ref().map(|ix| ix.info),
            GeometryStorage::Dedicated { index, .. } => index.as_ref().map(|(_, info)| *info),
        })
    }

    pub fn aabb(&self, id: u32) -> Option<Aabb> {
        self.records.get(&id).filter(|r| r.alive).map(|r| r.aabb)
    }

    pub fn maybe_compact_all(
        &mut self,
        frame_index: u64,
        threshold: f32,
        slack_ratio: f32,
        min_dead_bytes: u64,
    ) -> bool {
        let mut did = false;
        did |= self
            .index_u32
            .maybe_compact(frame_index, threshold, slack_ratio, min_dead_bytes);
        for p in &mut self.streams {
            did |= p.maybe_compact(frame_index, threshold, slack_ratio, min_dead_bytes);
        }
        did
    }
}

pub(crate) fn align4(n: u64) -> u64 {
    (n + 3) & !3
}

pub(crate) fn pad_to_4(v: &mut Vec<u8>) {
    let target = align4(v.len() as u64) as usize;
    while v.len() < target {
        v.push(0);
    }
}
