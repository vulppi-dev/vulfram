// GPU Arena Allocator (wgpu::Buffer suballocation)
//
// Key properties:
// - Suballocates linear slices from a growable GPU buffer.
// - Returns generation-checked handles (stale handle safety).
// - Freed allocations become "dead" space; optional compaction packs live allocations.
// - Old buffers are kept alive for `keep_frames` frames after resize/compaction (deferred drop).
//
// Notes:
// - Alignment is configurable; default is 4 bytes (sufficient for vertex/index buffers and COPY ops).
// - This allocator is byte-based. Higher-level systems (VertexAllocatorSystem) can plan growth in
//   element-count units by converting (count * stride) => bytes.
//
// Suggested module path: crate::core::alloc::arena
// ------------------------------------------------------------------------------

use std::ops::Range;

use bytemuck::Pod;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

// -----------------------------------------------------------------------------
// Public types
// -----------------------------------------------------------------------------

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

#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    pub capacity_bytes: u64,
    pub cursor_bytes: u64,
    pub live_bytes: u64,
    pub dead_bytes: u64,
    pub allocation_count: u32,
    pub free_handle_count: u32,
}

// -----------------------------------------------------------------------------
// Internal bookkeeping
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// ArenaAllocator
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct ArenaAllocator {
    // GPU state
    buffer: Buffer,
    usage: BufferUsages,
    capacity_bytes: u64,
    cursor: u64,

    // Configuration
    align: u64,
    min_capacity_bytes: u64,

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
    /// Default alignment (bytes).
    pub const DEFAULT_ALIGN: u64 = 4;

    /// Extra headroom used as a minimum in compaction shrink decisions.
    /// (This is a floor, not an addition; compaction can still end up larger due to live bytes.)
    const COMPACT_FLOOR_BYTES: u64 = 1024 * 24; // 24 KB

    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /// Create a new arena allocator with default alignment (4 bytes).
    pub fn new(
        device: &Device,
        queue: &Queue,
        initial_capacity_bytes: u64,
        usage: BufferUsages,
        label: Option<&'static str>,
    ) -> Self {
        Self::with_alignment(
            device,
            queue,
            initial_capacity_bytes,
            usage,
            label,
            Self::DEFAULT_ALIGN,
        )
    }

    /// Create a new arena allocator with an explicit alignment.
    ///
    /// `align` must be a power-of-two for the fast `align_up` implementation.
    pub fn with_alignment(
        device: &Device,
        queue: &Queue,
        initial_capacity_bytes: u64,
        usage: BufferUsages,
        label: Option<&'static str>,
        align: u64,
    ) -> Self {
        assert!(
            initial_capacity_bytes > 0,
            "initial_capacity_bytes must be > 0"
        );
        assert!(align.is_power_of_two(), "align must be power-of-two");
        assert!(align >= 4, "align should be >= 4 for wgpu-friendly offsets");

        let usage = usage | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;

        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size: initial_capacity_bytes,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            usage,
            capacity_bytes: initial_capacity_bytes,
            cursor: 0,
            align,
            min_capacity_bytes: initial_capacity_bytes,
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

    // -------------------------------------------------------------------------
    // Accessors / stats
    // -------------------------------------------------------------------------

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

    #[inline]
    pub fn alignment(&self) -> u64 {
        self.align
    }

    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            capacity_bytes: self.capacity_bytes,
            cursor_bytes: self.cursor,
            live_bytes: self.live_bytes,
            dead_bytes: self.dead_bytes,
            allocation_count: self.records.len() as u32,
            free_handle_count: self.free_indices.len() as u32,
        }
    }

    // -------------------------------------------------------------------------
    // Frame lifecycle / deferred drop
    // -------------------------------------------------------------------------

    pub fn set_keep_frames(&mut self, frames: u64) {
        self.keep_frames = frames.max(1);
    }

    /// Call once per frame to release old buffers that are safe to drop.
    pub fn begin_frame(&mut self, frame_index: u64) {
        self.garbage.retain(|g| g.retire_after_frame > frame_index);
    }

    // -------------------------------------------------------------------------
    // Slice resolution
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Capacity management (byte-based, used by higher-level element planning)
    // -------------------------------------------------------------------------

    /// Ensure the backing buffer can hold at least `required_capacity_bytes`.
    /// This does not change the cursor; it only grows the underlying buffer if needed.
    pub fn ensure_capacity_bytes(&mut self, required_capacity_bytes: u64) {
        if required_capacity_bytes <= self.capacity_bytes {
            return;
        }
        self.grow_to_fit(required_capacity_bytes);
    }

    /// Convenience helper: ensure capacity based on element count and stride.
    /// `required_bytes = align_up(required_elems * stride_bytes)`
    pub fn ensure_capacity_elems(&mut self, required_elems: u64, stride_bytes: u64) {
        assert!(stride_bytes > 0);
        let required = required_elems
            .checked_mul(stride_bytes)
            .expect("required capacity overflow");
        self.ensure_capacity_bytes(Self::align_up_to(required, self.align));
    }

    // -------------------------------------------------------------------------
    // Allocation
    // -------------------------------------------------------------------------

    /// Allocate a slice of `size_bytes` (must be multiple of `align`).
    pub fn allocate(&mut self, size_bytes: u64) -> AllocHandle {
        assert!(size_bytes > 0, "cannot allocate 0 bytes");
        assert!(
            size_bytes % self.align == 0,
            "size must be multiple of alignment"
        );

        let offset = Self::align_up_to(self.cursor, self.align);
        let end = offset.checked_add(size_bytes).expect("arena overflow");

        if end > self.capacity_bytes {
            self.grow_to_fit(end);
        }

        self.cursor = end;
        self.live_bytes = self
            .live_bytes
            .checked_add(size_bytes)
            .expect("live overflow");

        if let Some(idx) = self.free_indices.pop() {
            let rec = &mut self.records[idx as usize];
            rec.generation = rec.generation.wrapping_add(1);
            rec.alive = true;
            rec.offset = offset;
            rec.size = size_bytes;

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
                size: size_bytes,
            });
            AllocHandle {
                index: idx,
                generation: 1,
            }
        }
    }

    /// Allocate based on element count and stride, rounded up to alignment.
    pub fn allocate_elems(&mut self, elem_count: u64, stride_bytes: u64) -> AllocHandle {
        assert!(elem_count > 0);
        assert!(stride_bytes > 0);
        let raw = elem_count
            .checked_mul(stride_bytes)
            .expect("allocation overflow");
        let size = Self::align_up_to(raw, self.align);
        self.allocate(size)
    }

    // -------------------------------------------------------------------------
    // Writes
    // -------------------------------------------------------------------------

    /// Write raw bytes into an existing allocation.
    pub fn write_bytes(&self, handle: AllocHandle, data: &[u8]) {
        let slice = self.slice(handle);
        assert!(
            data.len() as u64 <= slice.size,
            "data does not fit allocation"
        );
        self.queue.write_buffer(&self.buffer, slice.offset, data);
    }

    /// Allocate and write raw bytes. Input length must already satisfy alignment.
    pub fn allocate_and_write(&mut self, data: &[u8]) -> AllocHandle {
        assert!(!data.is_empty(), "cannot allocate empty data");
        assert!(
            (data.len() as u64) % self.align == 0,
            "data length must be multiple of alignment"
        );

        let h = self.allocate(data.len() as u64);
        self.write_bytes(h, data);
        h
    }

    /// Allocate and write raw bytes, padding with zeros up to alignment automatically.
    /// This is often convenient for index/vertex streams that are not naturally multiple-of-4.
    pub fn allocate_and_write_padded(&mut self, data: &[u8]) -> AllocHandle {
        assert!(!data.is_empty(), "cannot allocate empty data");

        let padded_len = Self::align_up_to(data.len() as u64, self.align) as usize;
        if padded_len == data.len() {
            return self.allocate_and_write(data);
        }

        let mut tmp = Vec::with_capacity(padded_len);
        tmp.extend_from_slice(data);
        tmp.resize(padded_len, 0u8);

        let h = self.allocate(padded_len as u64);
        self.write_bytes(h, &tmp);
        h
    }

    /// Allocate and write a POD slice.
    pub fn allocate_and_write_pod_slice<T: Pod>(&mut self, values: &[T]) -> AllocHandle {
        assert!(!values.is_empty(), "cannot allocate empty slice");
        let bytes = bytemuck::cast_slice(values);
        self.allocate_and_write_padded(bytes)
    }

    // -------------------------------------------------------------------------
    // Free
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Compaction
    // -------------------------------------------------------------------------

    /// If dead space ratio exceeds `threshold`, compact into a new buffer with `slack_ratio` headroom.
    ///
    /// Returns true if compaction happened.
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
        // Collect live allocations, sorted by old offset (better copy pattern).
        let mut alive_indices: Vec<u32> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| if r.alive { Some(i as u32) } else { None })
            .collect();
        alive_indices.sort_by_key(|&i| self.records[i as usize].offset);

        // Decide new capacity: live + slack, rounded up to pow2, not below min_capacity_bytes.
        let live = self.live_bytes.max(1);
        let target = ((live as f32) * (1.0 + slack_ratio.max(0.0))) as u64;

        let floor = self
            .min_capacity_bytes
            .max(Self::COMPACT_FLOOR_BYTES)
            .max(self.align);

        let new_capacity = Self::next_pow2(target.max(floor));

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

        // Copy live slices, update offsets
        let mut new_cursor: u64 = 0;

        for idx in alive_indices {
            let rec = &mut self.records[idx as usize];
            let new_offset = Self::align_up_to(new_cursor, self.align);

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

        // Defer drop old buffer
        let old = std::mem::replace(&mut self.buffer, new_buffer);
        self.garbage.push(GarbageEntry {
            buffer: old,
            retire_after_frame: frame_index + self.keep_frames,
        });

        self.capacity_bytes = new_capacity;
        self.cursor = new_cursor;

        // After compaction, dead space has been physically removed.
        self.dead_bytes = 0;
    }

    // -------------------------------------------------------------------------
    // Resize / grow
    // -------------------------------------------------------------------------

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

        // Copy all bytes up to cursor (covers all live allocations).
        let copy_bytes = self.cursor;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ArenaAllocator resize encoder"),
            });

        encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, copy_bytes);
        self.queue.submit(Some(encoder.finish()));

        // Note: we do NOT defer-drop here because the old buffer is replaced immediately.
        // wgpu keeps it alive as long as needed; compaction uses explicit deferred drop.
        // If you prefer strict symmetry, you can also defer-drop resizes.
        self.buffer = new_buffer;
        self.capacity_bytes = new_capacity;
    }

    // -------------------------------------------------------------------------
    // Utilities
    // -------------------------------------------------------------------------

    #[inline]
    fn align_up_to(value: u64, align: u64) -> u64 {
        debug_assert!(align.is_power_of_two());
        (value + (align - 1)) & !(align - 1)
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
