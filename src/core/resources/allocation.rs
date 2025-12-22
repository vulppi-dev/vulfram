// MARK: - Uniform Buffer Pool

use bytemuck::{Pod, bytes_of};
use std::marker::PhantomData;
use std::ops::Range;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

/// Pool of uniform buffer entries with automatic capacity management.
/// Each entry has the same type T and occupies `sizeof<T>` bytes with proper alignment.
/// Automatically resizes when writing beyond current capacity.
#[derive(Debug)]
pub struct UniformBufferPool<T: Pod> {
    buffer: Buffer,
    capacity: u32,
    item_size: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _phantom: PhantomData<T>,
}

impl<T: Pod> UniformBufferPool<T> {
    // MARK: Constructor

    pub fn new(device: &Device, queue: &Queue, initial_capacity: Option<u32>) -> Self {
        let capacity = initial_capacity.unwrap_or(4);
        let item_size = std::mem::size_of::<T>() as u64;

        assert!(item_size > 0, "item_size must be greater than 0");

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
            _phantom: PhantomData,
        }
    }

    // MARK: Public Methods

    /// Write raw bytes to a specific index in the pool.
    /// Automatically resizes if index exceeds current capacity.
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

    /// Write a typed value to a specific index in the pool.
    /// Automatically resizes if index exceeds current capacity.
    pub fn write(&mut self, index: u32, value: &T) {
        let data = bytes_of(value);
        self.write_bytes(index, data);
    }

    /// Write multiple values starting at a specific index.
    /// Automatically resizes if needed.
    pub fn write_slice(&mut self, start_index: u32, values: &[T]) {
        if values.is_empty() {
            return;
        }

        let end_index = start_index + values.len() as u32;
        if end_index > self.capacity {
            self.scale_to_capacity(end_index);
        }

        for (i, value) in values.iter().enumerate() {
            let index = start_index + i as u32;
            let offset = index as u64 * self.item_size;
            self.queue
                .write_buffer(&self.buffer, offset, bytes_of(value));
        }
    }

    /// Get current capacity (number of entries that can be stored).
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Get size in bytes of each entry.
    pub fn item_size(&self) -> u64 {
        self.item_size
    }

    /// Get reference to underlying GPU buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get byte offset for a specific index.
    pub fn get_offset(&self, index: u32) -> u64 {
        index as u64 * self.item_size
    }

    /// Get total buffer size in bytes.
    pub fn buffer_size(&self) -> u64 {
        self.capacity as u64 * self.item_size
    }

    // MARK: Private Methods

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

        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    fn calculate_next_capacity(&self, required_capacity: u32) -> u32 {
        let mut new_capacity = self.capacity.max(1) * 2;

        while new_capacity < required_capacity {
            new_capacity = new_capacity.checked_mul(2).expect("capacity overflow");
        }

        new_capacity
    }
}

// MARK: - Arena Allocator

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllocHandle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ArenaSlice {
    pub offset: u64,
    pub size: u64,
}
impl ArenaSlice {
    #[inline]
    pub fn range(&self) -> Range<u64> {
        self.offset..(self.offset + self.size)
    }
}

#[derive(Debug, Clone)]
struct AllocRecord {
    generation: u32,
    alive: bool,
    offset: u64,
    size: u64,
}

#[derive(Debug)]
struct GarbageEntry {
    buffer: Buffer,
    retire_after_frame: u64,
}

#[derive(Debug)]
pub struct ArenaAllocator {
    // GPU State
    buffer: Buffer,
    usage: BufferUsages,
    capacity_bytes: u64,
    cursor: u64,

    // Bookkeeping
    records: Vec<AllocRecord>,
    free_indices: Vec<u32>,
    live_bytes: u64,
    dead_bytes: u64,

    // Deferred drop
    garbage: Vec<GarbageEntry>,
    keep_frames: u64,

    // WGPU handles
    device: wgpu::Device,
    queue: wgpu::Queue,
    label: Option<&'static str>,
}

impl ArenaAllocator {
    const ALIGN: u64 = 4;
    const EXTRA: u64 = 1024 * 24; // 2 KB extra on resize

    pub fn new(
        device: &Device,
        queue: &Queue,
        initial_capacity_bytes: u64,
        usage: BufferUsages,
        label: Option<&'static str>,
    ) -> Self {
        assert!(initial_capacity_bytes > 0);

        let usage = usage | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: label,
            size: initial_capacity_bytes,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            usage,
            capacity_bytes: initial_capacity_bytes,
            cursor: 0,
            records: Vec::new(),
            free_indices: Vec::new(),
            live_bytes: 0,
            dead_bytes: 0,
            garbage: Vec::new(),
            keep_frames: 3,
            device: device.clone(),
            queue: queue.clone(),
            label,
        }
    }

    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    #[inline]
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }

    #[inline]
    pub fn live_bytes(&self) -> u64 {
        self.live_bytes
    }

    #[inline]
    pub fn dead_bytes(&self) -> u64 {
        self.dead_bytes
    }

    pub fn set_keep_frames(&mut self, frames: u64) {
        self.keep_frames = frames.max(1);
    }

    /// Deve ser chamado 1x por frame para liberar buffers antigos já seguros.
    pub fn begin_frame(&mut self, frame_index: u64) {
        self.garbage.retain(|g| g.retire_after_frame > frame_index);
    }

    pub fn slice(&self, handle: AllocHandle) -> ArenaSlice {
        let rec = self
            .records
            .get(handle.index as usize)
            .unwrap_or_else(|| panic!("invalid handle index"));

        assert!(
            rec.generation == handle.generation,
            "stale handle generation"
        );
        assert!(rec.alive, "handle was freed");

        ArenaSlice {
            offset: rec.offset,
            size: rec.size,
        }
    }

    pub fn allocate(&mut self, size: u64) -> AllocHandle {
        assert!(size > 0);
        assert!(size % Self::ALIGN == 0, "size must be multiple of 4");

        let offset = Self::align_up(self.cursor);
        let end = offset.checked_add(size).expect("arena overflow");

        if end > self.capacity_bytes {
            self.grow_to_fit(end);
        }

        self.cursor = end;
        self.live_bytes = self.live_bytes.checked_add(size).expect("live overflow");

        if let Some(idx) = self.free_indices.pop() {
            let rec = &mut self.records[idx as usize];
            rec.generation = rec.generation.wrapping_add(1);
            rec.alive = true;
            rec.offset = offset;
            rec.size = size;
            AllocHandle {
                index: idx,
                generation: rec.generation,
            }
        } else {
            let idx = self.records.len() as u32;
            self.records.push(AllocRecord {
                generation: 1,
                alive: true,
                offset,
                size,
            });
            AllocHandle {
                index: idx,
                generation: 1,
            }
        }
    }

    pub fn write_bytes(&self, handle: AllocHandle, data: &[u8]) {
        let slice = self.slice(handle);
        assert!(data.len() as u64 <= slice.size);
        self.queue.write_buffer(&self.buffer, slice.offset, data);
    }

    pub fn allocate_and_write(&mut self, data: &[u8]) -> AllocHandle {
        assert!(!data.is_empty());
        assert!(
            (data.len() as u64) % Self::ALIGN == 0,
            "data size must be multiple of 4"
        );

        let h = self.allocate(data.len() as u64);
        self.write_bytes(h, data);
        h
    }

    pub fn allocate_and_write_pod_slice<T: Pod>(&mut self, values: &[T]) -> AllocHandle {
        if values.is_empty() {
            panic!("cannot allocate empty slice");
        }

        let bytes = bytemuck::cast_slice(values);

        assert!(
            (bytes.len() as u64) % Self::ALIGN == 0,
            "POD slice size must be multiple of 4 bytes"
        );

        self.allocate_and_write(bytes)
    }

    pub fn free(&mut self, handle: AllocHandle) {
        let idx = handle.index as usize;
        let rec = self
            .records
            .get_mut(idx)
            .unwrap_or_else(|| panic!("invalid handle index"));

        assert!(
            rec.generation == handle.generation,
            "stale handle generation"
        );
        assert!(rec.alive, "double free");

        rec.alive = false;

        self.dead_bytes = self
            .dead_bytes
            .checked_add(rec.size)
            .expect("dead overflow");
        self.live_bytes = self
            .live_bytes
            .checked_sub(rec.size)
            .expect("live underflow");

        self.free_indices.push(handle.index);
    }

    pub fn maybe_compact(
        &mut self,
        frame_index: u64,
        threshold: f32,
        slack_ratio: f32,
        min_dead_bytes: u64,
    ) -> bool {
        if self.capacity_bytes == 0 {
            return false;
        }

        if self.dead_bytes < min_dead_bytes {
            return false;
        }

        let dead_ratio = (self.dead_bytes as f64) / (self.capacity_bytes as f64);
        if dead_ratio < threshold as f64 {
            return false;
        }

        self.compact(frame_index, slack_ratio);
        true
    }

    fn compact(&mut self, frame_index: u64, slack_ratio: f32) {
        // Junta vivos, ordenados por offset antigo (melhor padrão de cópia)
        let mut alive_indices: Vec<u32> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| if r.alive { Some(i as u32) } else { None })
            .collect();
        alive_indices.sort_by_key(|&i| self.records[i as usize].offset);

        // Novo tamanho: live + folga, arredondado para pow2
        let live = self.live_bytes.max(1);
        let target = ((live as f32) * (1.0 + slack_ratio.max(0.0))) as u64;
        let new_capacity = Self::next_pow2(target.max(Self::EXTRA)); // evita buffers muito pequenos

        let new_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("ArenaAllocator (compacted)"),
            size: new_capacity,
            usage: self.usage,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ArenaAllocator compaction encoder"),
            });

        // Copia vivos e atualiza offsets
        let mut new_cursor: u64 = 0;

        for idx in alive_indices {
            let rec = &mut self.records[idx as usize];
            let new_offset = Self::align_up(new_cursor);

            encoder.copy_buffer_to_buffer(
                &self.buffer,
                rec.offset,
                &new_buffer,
                new_offset,
                rec.size,
            );

            rec.offset = new_offset;
            new_cursor = new_offset + rec.size;
        }

        self.queue.submit(Some(encoder.finish()));

        // Deferred drop do buffer antigo
        let old = std::mem::replace(&mut self.buffer, new_buffer);
        self.garbage.push(GarbageEntry {
            buffer: old,
            retire_after_frame: frame_index + self.keep_frames,
        });

        self.capacity_bytes = new_capacity;
        self.cursor = new_cursor;

        // Após compactar, tudo que era "morto" saiu fisicamente
        self.dead_bytes = 0;
    }

    fn grow_to_fit(&mut self, required_end: u64) {
        let mut new_capacity = self.capacity_bytes.max(1) * 2;
        while new_capacity < required_end {
            new_capacity = new_capacity.checked_mul(2).expect("capacity overflow");
        }

        let new_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("ArenaAllocator (resized)"),
            size: new_capacity,
            usage: self.usage,
            mapped_at_creation: false,
        });

        let copy_bytes = self.cursor;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ArenaAllocator resize encoder"),
            });

        encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, copy_bytes);
        self.queue.submit(Some(encoder.finish()));

        self.buffer = new_buffer;
        self.capacity_bytes = new_capacity;
    }

    #[inline]
    fn align_up(value: u64) -> u64 {
        (value + (Self::ALIGN - 1)) & !(Self::ALIGN - 1)
    }

    #[inline]
    fn next_pow2(mut v: u64) -> u64 {
        if v <= 1 {
            return 1;
        }
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v |= v >> 32;
        v + 1
    }
}
