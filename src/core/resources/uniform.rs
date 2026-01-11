// MARK: - Uniform Buffer Pool

use bytemuck::{Pod, bytes_of};
use std::marker::PhantomData;

// -----------------------------------------------------------------------------
// Internal types
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct GarbageEntry {
    _buffer: wgpu::Buffer,
    retire_after_frame: u64,
}

// -----------------------------------------------------------------------------
// UniformBufferPool
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct UniformBufferPool<T: Pod> {
    buffer: wgpu::Buffer,
    capacity: u32,
    item_size: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,

    // Deferred drop
    garbage: Vec<GarbageEntry>,
    keep_frames: u64,

    version: u64,

    _phantom: PhantomData<T>,
}

impl<T: Pod> UniformBufferPool<T> {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        initial_capacity: Option<u32>,
        alignment: u64,
    ) -> Self {
        let capacity = initial_capacity.unwrap_or(4);
        let raw_item_size = std::mem::size_of::<T>() as u64;

        assert!(raw_item_size > 0, "item_size must be greater than 0");

        // Align item size to required alignment
        let item_size = if alignment > 0 {
            (raw_item_size + alignment - 1) & !(alignment - 1)
        } else {
            raw_item_size
        };

        let buffer_size = capacity as u64 * item_size;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UniformBufferPool"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            item_size,
            device: device.clone(),
            queue: queue.clone(),
            garbage: Vec::new(),
            keep_frames: 3,
            version: 0,
            _phantom: PhantomData,
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn write_bytes(&mut self, index: u32, data: &[u8]) {
        assert!(
            data.len() as u64 <= self.item_size,
            "data size exceeds item_size"
        );

        if index + 1 > self.capacity {
            self.scale_to_capacity(index + 1);
        }

        let offset = index as u64 * self.item_size;
        self.queue.write_buffer(&self.buffer, offset, data);
    }

    pub fn write(&mut self, index: u32, value: &T) {
        let data = bytes_of(value);
        self.write_bytes(index, data);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn get_offset(&self, index: u32) -> u64 {
        index as u64 * self.item_size
    }

    // -------------------------------------------------------------------------
    // Frame lifecycle / deferred drop
    // -------------------------------------------------------------------------

    /// Call once per frame to release old buffers that are safe to drop.
    pub fn begin_frame(&mut self, frame_index: u64) {
        // Set retire frame for new garbage entries
        for g in &mut self.garbage {
            if g.retire_after_frame == 0 {
                g.retire_after_frame = frame_index + self.keep_frames;
            }
        }

        // Remove buffers that are safe to drop
        self.garbage.retain(|g| g.retire_after_frame > frame_index);
    }

    // -------------------------------------------------------------------------
    // Resize
    // -------------------------------------------------------------------------

    fn scale_to_capacity(&mut self, required_capacity: u32) {
        if required_capacity <= self.capacity {
            return;
        }

        let new_capacity = self.calculate_next_capacity(required_capacity);
        let new_size = new_capacity as u64 * self.item_size;

        let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UniformBufferPool (resized)"),
            size: new_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UniformBufferPool resize encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            self.capacity as u64 * self.item_size,
        );

        self.queue.submit(Some(encoder.finish()));

        // Deferred drop: keep old buffer alive for keep_frames
        // This ensures GPU commands referencing it have completed
        let old = std::mem::replace(&mut self.buffer, new_buffer);
        self.garbage.push(GarbageEntry {
            _buffer: old,
            retire_after_frame: 0, // Will be set by first begin_frame call
        });

        self.capacity = new_capacity;
        self.version += 1;
    }

    fn calculate_next_capacity(&self, required_capacity: u32) -> u32 {
        let mut new_capacity = self.capacity.max(1) * 2;

        while new_capacity < required_capacity {
            new_capacity = new_capacity.checked_mul(2).expect("capacity overflow");
        }

        new_capacity
    }
}
