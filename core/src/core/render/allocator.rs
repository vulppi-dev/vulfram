use std::collections::HashMap;

/// Buffer allocator with free space reuse
///
/// Manages allocation and deallocation of space within a shared buffer,
/// maintaining a free-list of available slots for efficient reuse.
#[derive(Debug)]
pub struct BufferAllocator {
    /// Next available offset at the end (only used if no free slots)
    next_offset: u64,
    /// Active allocations: id → (offset, size)
    allocations: HashMap<u32, (u64, u64)>,
    /// Free slots that can be reused: (offset, size)
    /// Kept sorted by offset for efficient coalescing
    free_slots: Vec<(u64, u64)>,
    /// Total buffer capacity
    capacity: u64,
}

impl BufferAllocator {
    /// Create a new buffer allocator with the given initial capacity
    pub fn new(initial_capacity: u64) -> Self {
        Self {
            next_offset: 0,
            allocations: HashMap::new(),
            free_slots: Vec::new(),
            capacity: initial_capacity,
        }
    }

    /// Allocate space for a resource
    ///
    /// Returns (offset, needs_buffer_recreation)
    /// Uses first-fit strategy: finds the first free slot that fits the requested size
    pub fn allocate(&mut self, id: u32, size: u64) -> Result<(u64, bool), String> {
        // Alignment to 256 bytes (required by WGPU for uniform buffers)
        let aligned_size = (size + 255) & !255;

        // Try to find a suitable free slot (first-fit strategy)
        for i in 0..self.free_slots.len() {
            let (offset, slot_size) = self.free_slots[i];

            if slot_size >= aligned_size {
                // Found a suitable slot!
                self.allocations.insert(id, (offset, aligned_size));

                // Remove or shrink the free slot
                if slot_size == aligned_size {
                    // Perfect fit, remove the slot
                    self.free_slots.remove(i);
                } else {
                    // Partial use, shrink the slot
                    self.free_slots[i] = (offset + aligned_size, slot_size - aligned_size);
                }

                return Ok((offset, false));
            }
        }

        // No suitable free slot, allocate at the end
        let offset = self.next_offset;
        self.next_offset += aligned_size;
        self.allocations.insert(id, (offset, aligned_size));

        // Check if we need to grow the buffer
        let needs_recreation = self.next_offset > self.capacity;
        if needs_recreation {
            // Grow by 2x or at least to fit
            self.capacity = std::cmp::max(self.capacity * 2, self.next_offset);
        }

        Ok((offset, needs_recreation))
    }

    /// Free space used by a resource
    ///
    /// Marks the space as available for reuse and attempts to coalesce
    /// with adjacent free slots to reduce fragmentation
    pub fn deallocate(&mut self, id: u32) {
        if let Some((offset, size)) = self.allocations.remove(&id) {
            // Add to free slots
            self.free_slots.push((offset, size));

            // Sort by offset for coalescing
            self.free_slots.sort_by_key(|&(off, _)| off);

            // Coalesce adjacent free slots
            self.coalesce_free_slots();
        }
    }

    /// Merge adjacent free slots to reduce fragmentation
    fn coalesce_free_slots(&mut self) {
        if self.free_slots.len() < 2 {
            return;
        }

        let mut i = 0;
        while i < self.free_slots.len() - 1 {
            let (offset1, size1) = self.free_slots[i];
            let (offset2, size2) = self.free_slots[i + 1];

            // If slots are adjacent, merge them
            if offset1 + size1 == offset2 {
                self.free_slots[i] = (offset1, size1 + size2);
                self.free_slots.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }

    /// Get current buffer capacity
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut allocator = BufferAllocator::new(1024);

        let (offset1, needs_recreation) = allocator.allocate(1, 128).unwrap();
        assert_eq!(offset1, 0);
        assert_eq!(needs_recreation, false);

        let (offset2, needs_recreation) = allocator.allocate(2, 128).unwrap();
        assert_eq!(offset2, 256); // Aligned to 256
        assert_eq!(needs_recreation, false);
    }

    #[test]
    fn test_deallocation_and_reuse() {
        let mut allocator = BufferAllocator::new(1024);

        allocator.allocate(1, 128).unwrap();
        let (offset2, _) = allocator.allocate(2, 128).unwrap();
        allocator.allocate(3, 128).unwrap();

        // Deallocate middle allocation
        allocator.deallocate(2);

        // Next allocation should reuse the freed slot
        let (offset_reused, _) = allocator.allocate(4, 128).unwrap();
        assert_eq!(offset_reused, offset2);
    }

    #[test]
    fn test_coalescing() {
        let mut allocator = BufferAllocator::new(1024);

        allocator.allocate(1, 128).unwrap();
        allocator.allocate(2, 128).unwrap();
        allocator.allocate(3, 128).unwrap();

        // Deallocate adjacent allocations
        allocator.deallocate(1);
        allocator.deallocate(2);

        // Should have coalesced into one large slot
        assert_eq!(allocator.free_slots.len(), 1);
        assert_eq!(allocator.free_slots[0].1, 512); // 256 + 256
    }
}
