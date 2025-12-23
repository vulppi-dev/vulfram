use std::collections::HashMap;
use std::ops::Range;
use wgpu::{Buffer, BufferDescriptor, Device, Queue, RenderPass};

mod arena;
mod bind_cache;
mod defaults;
mod pools;
mod storage;
mod types;

use arena::*;
use bind_cache::*;
use defaults::*;
use pools::*;
use storage::*;
use types::*;

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

    index_u32: IndexPool,
    streams: [StreamPool; STREAM_COUNT],

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
    pub fn new(device: &Device, queue: &Queue, cfg: VertexAllocatorConfig) -> Self {
        let index_u32 = IndexPool::new(device, queue, cfg);

        let streams = [
            StreamPool::new(device, queue, cfg, VertexStream::Position),
            StreamPool::new(device, queue, cfg, VertexStream::Normal),
            StreamPool::new(device, queue, cfg, VertexStream::Tangent),
            StreamPool::new(device, queue, cfg, VertexStream::Color0),
            StreamPool::new(device, queue, cfg, VertexStream::UV0),
            StreamPool::new(device, queue, cfg, VertexStream::UV1),
            StreamPool::new(device, queue, cfg, VertexStream::Joints),
            StreamPool::new(device, queue, cfg, VertexStream::Weights),
        ];

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
        self.index_u32.arena.begin_frame(frame_index);
        for p in &mut self.streams {
            p.arena.begin_frame(frame_index);
        }
        self.bind_cache.reset();
    }

    pub fn create_geometry(
        &mut self,
        id: u32,
        mut input: Vec<(GeometryPrimitiveType, Vec<u8>)>,
    ) -> Result<(), VertexAllocError> {
        let mut index_bytes: Option<Vec<u8>> = None;
        let mut stream_bytes: [Option<Vec<u8>>; STREAM_COUNT] = [(); STREAM_COUNT].map(|_| None);
        let mut uv_seen = 0;
        let mut seen_prim: HashMap<u32, ()> = HashMap::new();

        for (prim, bytes) in input.drain(..) {
            match prim {
                GeometryPrimitiveType::Index => {
                    if index_bytes.is_some() {
                        return Err(VertexAllocError::DuplicateIndex);
                    }
                    index_bytes = Some(bytes);
                }
                GeometryPrimitiveType::Position => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::Position as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Position as usize] = Some(bytes);
                }
                GeometryPrimitiveType::Normal => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::Normal as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Normal as usize] = Some(bytes);
                }
                GeometryPrimitiveType::Tangent => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::Tangent as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Tangent as usize] = Some(bytes);
                }
                GeometryPrimitiveType::Color => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::Color as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Color0 as usize] = Some(bytes);
                }
                GeometryPrimitiveType::UV => {
                    if uv_seen >= 2 {
                        return Err(VertexAllocError::TooManyUVSets);
                    }
                    if uv_seen == 0 {
                        stream_bytes[VertexStream::UV0 as usize] = Some(bytes);
                    } else {
                        stream_bytes[VertexStream::UV1 as usize] = Some(bytes);
                    }
                    uv_seen += 1;
                }
                GeometryPrimitiveType::SkinJoints => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::SkinJoints as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Joints as usize] = Some(bytes);
                }
                GeometryPrimitiveType::SkinWeights => {
                    if seen_prim
                        .insert(GeometryPrimitiveType::SkinWeights as u32, ())
                        .is_some()
                    {
                        return Err(VertexAllocError::DuplicateStream(prim));
                    }
                    stream_bytes[VertexStream::Weights as usize] = Some(bytes);
                }
            }
        }

        let pos = stream_bytes[VertexStream::Position as usize]
            .as_ref()
            .ok_or(VertexAllocError::MissingPosition)?;

        let vertex_count = {
            let stride = VertexStream::Position.stride_bytes();
            if pos.len() % stride as usize != 0 {
                return Err(VertexAllocError::InvalidStride {
                    stream: VertexStream::Position,
                    byte_len: pos.len(),
                    stride,
                });
            }
            (pos.len() as u64 / stride) as u32
        };

        for s in all_streams() {
            if let Some(b) = stream_bytes[s as usize].as_ref() {
                let stride = s.stride_bytes();
                if b.len() % stride as usize != 0 {
                    return Err(VertexAllocError::InvalidStride {
                        stream: s,
                        byte_len: b.len(),
                        stride,
                    });
                }
                let count = (b.len() as u64 / stride) as u32;
                if count != vertex_count {
                    return Err(VertexAllocError::PositionCountMismatch {
                        expected: vertex_count,
                        got: count,
                        stream: s,
                    });
                }
            }
        }

        let index_info: Option<(Vec<u8>, IndexInfo)> = if let Some(b) = index_bytes.take() {
            if b.len() % 4 != 0 {
                return Err(VertexAllocError::InvalidIndexBytes {
                    byte_len: b.len(),
                    reason: "must be multiple of 4 (u32 format)",
                });
            }
            let count = (b.len() / 4) as u32;
            if count == 0 {
                return Err(VertexAllocError::InvalidIndexBytes {
                    byte_len: b.len(),
                    reason: "zero indices",
                });
            }
            Some((b, IndexInfo { count }))
        } else {
            None
        };

        let mut total_aligned: u64 = 0;
        if let Some((b, _)) = index_info.as_ref() {
            total_aligned += align4(b.len() as u64);
        }
        for s in all_streams() {
            if let Some(b) = stream_bytes[s as usize].as_ref() {
                total_aligned += align4(b.len() as u64);
            }
        }

        let storage = if total_aligned > self.cfg.dedicated_threshold_bytes {
            self.create_dedicated(vertex_count, index_info, stream_bytes)?
        } else {
            self.create_pooled(vertex_count, index_info, stream_bytes)?
        };

        if let Some(rec) = self.records.get_mut(&id) {
            rec.alive = true;
            rec.storage = storage;
        } else {
            self.records.insert(
                id,
                GeometryRecord {
                    alive: true,
                    storage,
                },
            );
        }

        Ok(())
    }

    fn create_pooled(
        &mut self,
        vertex_count: u32,
        index_info: Option<(Vec<u8>, IndexInfo)>,
        mut stream_bytes: [Option<Vec<u8>>; STREAM_COUNT],
    ) -> Result<GeometryStorage, VertexAllocError> {
        self.ensure_default_capacity(vertex_count);

        let index_alloc = if let Some((mut bytes, info)) = index_info {
            pad_to_4(&mut bytes);
            let h = self.index_u32.arena.allocate(bytes.len() as u64);
            self.queue.write_buffer(
                self.index_u32.buffer(),
                self.index_u32.slice_range(h).start,
                &bytes,
            );
            Some(types::IndexAlloc { handle: h, info })
        } else {
            None
        };

        let mut handles: [Option<AllocHandle>; STREAM_COUNT] = [(); STREAM_COUNT].map(|_| None);

        for s in all_streams() {
            if let Some(mut bytes) = stream_bytes[s as usize].take() {
                pad_to_4(&mut bytes);
                let h = self.streams[s as usize].arena.allocate(bytes.len() as u64);
                self.queue.write_buffer(
                    self.streams[s as usize].buffer(),
                    self.streams[s as usize].slice_range(h).start,
                    &bytes,
                );
                handles[s as usize] = Some(h);
            }
        }

        Ok(GeometryStorage::Pooled {
            index: index_alloc,
            streams: handles,
            vertex_count,
        })
    }

    fn create_dedicated(
        &mut self,
        vertex_count: u32,
        index_info: Option<(Vec<u8>, IndexInfo)>,
        mut stream_bytes: [Option<Vec<u8>>; STREAM_COUNT],
    ) -> Result<GeometryStorage, VertexAllocError> {
        self.ensure_default_capacity(vertex_count);

        let mut cursor: u64 = 0;
        let mut index_range: Option<(Range<u64>, IndexInfo)> = None;
        let mut stream_ranges: [Option<Range<u64>>; STREAM_COUNT] =
            [(); STREAM_COUNT].map(|_| None);

        let mut index_bytes_opt = index_info.map(|(b, info)| (b, info));

        if let Some((mut idx_bytes, info)) = index_bytes_opt.take() {
            pad_to_4(&mut idx_bytes);
            let len = idx_bytes.len() as u64;
            let r = cursor..cursor + len;
            cursor += align4(len);
            index_range = Some((r, info));
            index_bytes_opt = Some((idx_bytes, info));
        }

        for s in all_streams() {
            if let Some(mut bytes) = stream_bytes[s as usize].take() {
                pad_to_4(&mut bytes);
                let len = bytes.len() as u64;
                let r = cursor..cursor + len;
                cursor += align4(len);
                stream_ranges[s as usize] = Some(r);
                stream_bytes[s as usize] = Some(bytes);
            }
        }

        let total_size = align4(cursor).max(4);
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Geometry(Dedicated)"),
            size: total_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        if let Some((idx_bytes, _info)) = index_bytes_opt.take() {
            if let Some((r, _)) = &index_range {
                self.queue.write_buffer(&buffer, r.start, &idx_bytes);
            }
        }

        for s in all_streams() {
            if let Some(bytes) = stream_bytes[s as usize].take() {
                if let Some(r) = &stream_ranges[s as usize] {
                    self.queue.write_buffer(&buffer, r.start, &bytes);
                }
            }
        }

        Ok(GeometryStorage::Dedicated {
            buffer,
            index: index_range,
            streams: stream_ranges,
            vertex_count,
        })
    }

    fn ensure_default_capacity(&mut self, vertex_count: u32) {
        self.default_normal
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_tangent
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_color0
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_uv0
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_uv1
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_joints
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
        self.default_weights
            .ensure_vertices(&self.device, &self.queue, self.cfg, vertex_count);
    }

    pub fn destroy_geometry(&mut self, id: u32) -> Result<(), VertexAllocError> {
        let rec = self
            .records
            .get_mut(&id)
            .ok_or(VertexAllocError::GeometryNotFound)?;

        if !rec.alive {
            return Err(VertexAllocError::GeometryNotFound);
        }

        match &rec.storage {
            GeometryStorage::Pooled { index, streams, .. } => {
                if let Some(ix) = index {
                    self.index_u32.arena.free(ix.handle);
                }
                for (i, h_opt) in streams.iter().enumerate() {
                    if let Some(h) = h_opt {
                        self.streams[i].arena.free(*h);
                    }
                }
            }
            GeometryStorage::Dedicated { .. } => {}
        }

        rec.alive = false;
        Ok(())
    }

    pub fn bind<'a>(
        &'a mut self,
        pass: &mut RenderPass<'a>,
        id: u32,
    ) -> Result<(), VertexAllocError> {
        let rec = self
            .records
            .get(&id)
            .ok_or(VertexAllocError::GeometryNotFound)?;

        if !rec.alive {
            return Err(VertexAllocError::GeometryNotFound);
        }

        let vertex_count = match &rec.storage {
            GeometryStorage::Pooled { vertex_count, .. } => *vertex_count,
            GeometryStorage::Dedicated { vertex_count, .. } => *vertex_count,
        };

        // Pre-collect all buffer references before mutating self
        struct BindData {
            index_buffer: Option<(*const Buffer, Range<u64>)>,
            vertex_buffers: Vec<(u32, *const Buffer, Range<u64>)>,
        }

        let bind_data = match &rec.storage {
            GeometryStorage::Pooled { index, streams, .. } => {
                let index_buffer = index.as_ref().map(|ix| {
                    let buf = self.index_u32.buffer() as *const Buffer;
                    let r = self.index_u32.slice_range(ix.handle);
                    (buf, r)
                });

                let mut vertex_buffers = Vec::new();
                for s in all_streams() {
                    let slot = s.slot();
                    let (buf_ptr, range) = if let Some(h) = streams[s as usize] {
                        let pool = &self.streams[s as usize];
                        (pool.buffer() as *const Buffer, pool.slice_range(h))
                    } else {
                        let (buf, r) = self.default_slice_for(s, vertex_count);
                        (buf as *const Buffer, r)
                    };
                    vertex_buffers.push((slot, buf_ptr, range));
                }

                BindData {
                    index_buffer,
                    vertex_buffers,
                }
            }
            GeometryStorage::Dedicated {
                buffer,
                index,
                streams,
                ..
            } => {
                let index_buffer = index.as_ref().map(|(r, _info)| {
                    let buf = buffer as *const Buffer;
                    (buf, r.clone())
                });

                let mut vertex_buffers = Vec::new();
                for s in all_streams() {
                    let slot = s.slot();
                    let (buf_ptr, range) = if let Some(r) = &streams[s as usize] {
                        (buffer as *const Buffer, r.clone())
                    } else {
                        let (buf, r) = self.default_slice_for(s, vertex_count);
                        (buf as *const Buffer, r)
                    };
                    vertex_buffers.push((slot, buf_ptr, range));
                }

                BindData {
                    index_buffer,
                    vertex_buffers,
                }
            }
        };

        // Now apply all binds with mutable access to self
        if let Some((buf_ptr, range)) = bind_data.index_buffer {
            let buf = unsafe { &*buf_ptr };
            self.set_index_cached(pass, buf, range);
        }

        for (slot, buf_ptr, range) in bind_data.vertex_buffers {
            let buf = unsafe { &*buf_ptr };
            self.set_vertex_cached(pass, slot, buf, range);
        }

        Ok(())
    }

    fn default_slice_for(&self, stream: VertexStream, vertex_count: u32) -> (&Buffer, Range<u64>) {
        let bytes = (vertex_count as u64) * stream.stride_bytes();
        let r = 0..bytes;

        match stream {
            VertexStream::Position => (
                &self.streams[VertexStream::Position as usize].arena.buffer(),
                0..0,
            ),
            VertexStream::Normal => (&self.default_normal.buffer, r),
            VertexStream::Tangent => (&self.default_tangent.buffer, r),
            VertexStream::Color0 => (&self.default_color0.buffer, r),
            VertexStream::UV0 => (&self.default_uv0.buffer, r),
            VertexStream::UV1 => (&self.default_uv1.buffer, r),
            VertexStream::Joints => (&self.default_joints.buffer, r),
            VertexStream::Weights => (&self.default_weights.buffer, r),
        }
    }

    fn set_index_cached(&mut self, pass: &mut RenderPass<'_>, buffer: &Buffer, range: Range<u64>) {
        let key = (buffer as *const Buffer as usize, range.clone());

        if let Some((b, r)) = &self.bind_cache.last_index {
            if *b == key.0 && r.start == key.1.start && r.end == key.1.end {
                return;
            }
        }

        pass.set_index_buffer(buffer.slice(range.clone()), wgpu::IndexFormat::Uint32);
        self.bind_cache.last_index = Some(key);
    }

    fn set_vertex_cached(
        &mut self,
        pass: &mut RenderPass<'_>,
        slot: u32,
        buffer: &Buffer,
        range: Range<u64>,
    ) {
        let i = slot as usize;
        let key = (buffer as *const Buffer as usize, range.clone());

        if let Some((b, r)) = &self.bind_cache.last_vertex[i] {
            if *b == key.0 && r.start == key.1.start && r.end == key.1.end {
                return;
            }
        }

        pass.set_vertex_buffer(slot, buffer.slice(range.clone()));
        self.bind_cache.last_vertex[i] = Some(key);
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

    pub fn vertex_count(&self, id: u32) -> Result<u32, VertexAllocError> {
        let rec = self
            .records
            .get(&id)
            .ok_or(VertexAllocError::GeometryNotFound)?;
        if !rec.alive {
            return Err(VertexAllocError::GeometryNotFound);
        }
        Ok(match &rec.storage {
            GeometryStorage::Pooled { vertex_count, .. } => *vertex_count,
            GeometryStorage::Dedicated { vertex_count, .. } => *vertex_count,
        })
    }

    pub fn maybe_compact_all(
        &mut self,
        frame_index: u64,
        threshold: f32,
        slack_ratio: f32,
        min_dead_bytes: u64,
    ) -> bool {
        let mut did = false;
        did |=
            self.index_u32
                .arena
                .maybe_compact(frame_index, threshold, slack_ratio, min_dead_bytes);
        for p in &mut self.streams {
            did |= p
                .arena
                .maybe_compact(frame_index, threshold, slack_ratio, min_dead_bytes);
        }
        did
    }
}
