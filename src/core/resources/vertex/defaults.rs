use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use super::types::{VertexAllocatorConfig, VertexStream};

// -----------------------------------------------------------------------------
// Default buffers (fallback), one per optional stream
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct DefaultStreamBuffer {
    pub stream: VertexStream,
    pub stride: u64,
    pub buffer: Buffer,
    pub capacity_vertices: u32,
}

impl DefaultStreamBuffer {
    pub fn new(device: &Device, cfg: VertexAllocatorConfig, stream: VertexStream) -> Self {
        let stride = stream.stride_bytes();
        let initial_bytes = cfg.min_pool_bytes.max(stride);
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(match stream {
                VertexStream::Normal => "Default(Normal)",
                VertexStream::Tangent => "Default(Tangent)",
                VertexStream::Color0 => "Default(Color0)",
                VertexStream::UV0 => "Default(UV0)",
                VertexStream::UV1 => "Default(UV1)",
                VertexStream::Joints => "Default(Joints)",
                VertexStream::Weights => "Default(Weights)",
                VertexStream::Position => "Default(Position) [invalid]",
            }),
            size: initial_bytes,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let capacity_vertices = (initial_bytes / stride).max(1) as u32;
        Self {
            stream,
            stride,
            buffer,
            capacity_vertices,
        }
    }

    pub fn ensure_vertices(
        &mut self,
        device: &Device,
        queue: &Queue,
        cfg: VertexAllocatorConfig,
        required_vertices: u32,
    ) {
        if required_vertices <= self.capacity_vertices {
            return;
        }

        let mut new_cap = self.capacity_vertices.max(1);
        while new_cap < required_vertices {
            new_cap = new_cap.saturating_mul(2);
        }

        let new_bytes = (new_cap as u64)
            .saturating_mul(self.stride)
            .max(cfg.min_pool_bytes);

        let new_cap_from_bytes = (new_bytes / self.stride).max(1) as u32;

        let new_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("DefaultStreamBuffer (resized)"),
            size: new_bytes,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fill = build_repeated_default_bytes(self.stream, self.stride, new_cap_from_bytes);
        queue.write_buffer(&new_buffer, 0, &fill);

        self.buffer = new_buffer;
        self.capacity_vertices = new_cap_from_bytes;
    }
}

pub fn build_repeated_default_bytes(
    stream: VertexStream,
    stride: u64,
    vertex_count: u32,
) -> Vec<u8> {
    let one = default_value_bytes(stream);
    debug_assert_eq!(one.len() as u64, stride);

    let total = (stride as usize).saturating_mul(vertex_count as usize);
    let mut out = Vec::with_capacity(total);

    for _ in 0..vertex_count {
        out.extend_from_slice(&one);
    }
    out
}

pub fn default_value_bytes(stream: VertexStream) -> Vec<u8> {
    match stream {
        VertexStream::Position => vec![0u8; 12],
        VertexStream::Normal => {
            let mut v = Vec::with_capacity(12);
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v
        }
        VertexStream::Tangent => {
            let mut v = Vec::with_capacity(16);
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v
        }
        VertexStream::Color0 => {
            let mut v = Vec::with_capacity(16);
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v
        }
        VertexStream::UV0 | VertexStream::UV1 => {
            let mut v = Vec::with_capacity(8);
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v
        }
        VertexStream::Joints => vec![0u8; 8],
        VertexStream::Weights => {
            let mut v = Vec::with_capacity(16);
            v.extend_from_slice(&1.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v
        }
    }
}

#[inline]
pub fn align4(v: u64) -> u64 {
    (v + 3) & !3
}

#[inline]
pub fn pad_to_4(bytes: &mut Vec<u8>) {
    let rem = bytes.len() & 3;
    if rem != 0 {
        let pad = 4 - rem;
        bytes.extend(std::iter::repeat(0u8).take(pad));
    }
}
