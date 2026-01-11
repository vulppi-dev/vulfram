use std::collections::HashMap;
use std::ops::Range;
use wgpu::{BufferDescriptor};
use crate::core::resources::geometry::Aabb;
use super::{
    VertexAllocatorSystem, VertexAllocError, GeometryPrimitiveType, 
    VertexStream, all_streams, GeometryRecord, GeometryStorage, 
    align4, pad_to_4, IndexInfo, AllocHandle, IndexAlloc
};

impl VertexAllocatorSystem {
    pub fn create_geometry(
        &mut self,
        id: u32,
        mut input: Vec<(GeometryPrimitiveType, Vec<u8>)>,
    ) -> Result<(), VertexAllocError> {
        let mut index_bytes: Option<Vec<u8>> = None;
        let mut stream_bytes: [Option<Vec<u8>>; 8] = [(); 8].map(|_| None);
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

        let aabb = Aabb::from_bytes(pos);

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

        // CRITICAL: If ID already exists, we must free old allocations before replacing
        if let Some(rec) = self.records.get_mut(&id) {
            if rec.alive {
                // Geometry still alive - free old allocations before replacing
                match &rec.storage {
                    GeometryStorage::Pooled { index, streams, .. } => {
                        if let Some(ix) = index {
                            self.index_u32.free(ix.handle);
                        }
                        for (i, h_opt) in streams.iter().enumerate() {
                            if let Some(h) = h_opt {
                                self.streams[i].free(*h);
                            }
                        }
                    }
                    GeometryStorage::Dedicated { .. } => {
                        // Dedicated buffer will be dropped when storage is replaced
                    }
                }
            }
            rec.alive = true;
            rec.storage = storage;
            rec.aabb = aabb;
        } else {
            self.records.insert(
                id,
                GeometryRecord {
                    alive: true,
                    storage,
                    aabb,
                },
            );
        }

        Ok(())
    }

    fn create_pooled(
        &mut self,
        vertex_count: u32,
        index_info: Option<(Vec<u8>, IndexInfo)>,
        mut stream_bytes: [Option<Vec<u8>>; 8],
    ) -> Result<GeometryStorage, VertexAllocError> {
        self.ensure_default_capacity(vertex_count);

        let index_alloc = if let Some((bytes, info)) = index_info {
            let h = self.index_u32.allocate_and_write_padded(&bytes);
            Some(IndexAlloc { handle: h, info })
        } else {
            None
        };

        let mut handles: [Option<AllocHandle>; 8] = [(); 8].map(|_| None);

        for s in all_streams() {
            if let Some(bytes) = stream_bytes[s as usize].take() {
                let h = self.streams[s as usize].allocate_and_write_padded(&bytes);
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
        mut stream_bytes: [Option<Vec<u8>>; 8],
    ) -> Result<GeometryStorage, VertexAllocError> {
        self.ensure_default_capacity(vertex_count);

        let mut cursor: u64 = 0;
        let mut index_range: Option<(Range<u64>, IndexInfo)> = None;
        let mut stream_ranges: [Option<Range<u64>>; 8] =
            [(); 8].map(|_| None);

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
            .remove(&id)
            .ok_or(VertexAllocError::GeometryNotFound)?;

        if !rec.alive {
            return Err(VertexAllocError::GeometryNotFound);
        }

        match rec.storage {
            GeometryStorage::Pooled { index, streams, .. } => {
                if let Some(ix) = index {
                    self.index_u32.free(ix.handle);
                }
                for (i, h_opt) in streams.iter().enumerate() {
                    if let Some(h) = h_opt {
                        self.streams[i].free(*h);
                    }
                }
            }
            GeometryStorage::Dedicated { .. } => {
                // Dedicated buffer will be dropped here automatically
            }
        }

        Ok(())
    }
}
