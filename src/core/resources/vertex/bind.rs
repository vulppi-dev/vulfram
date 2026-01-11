use super::{GeometryStorage, VertexAllocError, VertexAllocatorSystem, VertexStream, all_streams};
use std::ops::Range;
use wgpu::{Buffer, RenderPass};

impl VertexAllocatorSystem {
    pub fn bind<'a>(&mut self, pass: &mut RenderPass<'a>, id: u32) -> Result<(), VertexAllocError> {
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
                    let r = self.index_u32.slice(ix.handle).range();
                    (buf, r)
                });

                let mut vertex_buffers = Vec::new();
                for s in all_streams() {
                    let slot = s.slot();
                    let (buf_ptr, range) = if let Some(h) = streams[s as usize] {
                        let pool = &self.streams[s as usize];
                        (pool.buffer() as *const Buffer, pool.slice(h).range())
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
                &self.streams[VertexStream::Position as usize].buffer(),
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
}
