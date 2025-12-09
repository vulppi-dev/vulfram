use std::collections::HashMap;
use wgpu;

/// Manages allocation of offsets within shared uniform buffers
/// Uses a simple bump allocator strategy with alignment requirements
pub struct UniformBufferAllocator {
    /// Next available offset in bytes
    next_offset: u32,
    /// Minimum uniform buffer offset alignment (usually 256 bytes for most GPUs)
    alignment: u32,
    /// Track allocated ranges for potential deallocation (future optimization)
    allocations: HashMap<u32, u32>, // offset -> size
}

impl UniformBufferAllocator {
    /// Create a new allocator with the given alignment requirement
    pub fn new(alignment: u32) -> Self {
        Self {
            next_offset: 0,
            alignment,
            allocations: HashMap::new(),
        }
    }

    /// Allocate space for uniform data of given size
    /// Returns the aligned offset where data should be written
    pub fn allocate(&mut self, size: u32) -> u32 {
        // Align current offset to required alignment
        let aligned_offset = self.align_offset(self.next_offset);
        
        // Store allocation
        self.allocations.insert(aligned_offset, size);
        
        // Advance next_offset
        self.next_offset = aligned_offset + size;
        
        aligned_offset
    }

    /// Free allocation at given offset (marks space as reusable)
    /// Note: Current implementation doesn't reuse freed space (simple bump allocator)
    /// Future optimization: implement a proper free list allocator
    #[allow(dead_code)]
    pub fn free(&mut self, offset: u32) {
        self.allocations.remove(&offset);
    }

    /// Get total allocated size in bytes
    pub fn total_size(&self) -> u32 {
        self.next_offset
    }

    /// Reset allocator to initial state (useful when recreating buffers)
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.next_offset = 0;
        self.allocations.clear();
    }

    /// Align offset to required alignment
    fn align_offset(&self, offset: u32) -> u32 {
        let remainder = offset % self.alignment;
        if remainder == 0 {
            offset
        } else {
            offset + (self.alignment - remainder)
        }
    }
}

/// GPU buffer wrapper with allocator for dynamic uniform data
pub struct DynamicUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub allocator: UniformBufferAllocator,
    pub capacity: u32,
}

impl DynamicUniformBuffer {
    /// Create a new dynamic uniform buffer with initial capacity
    pub fn new(device: &wgpu::Device, alignment: u32, initial_capacity: u32, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: initial_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            allocator: UniformBufferAllocator::new(alignment),
            capacity: initial_capacity,
        }
    }

    /// Allocate space and return offset
    pub fn allocate(&mut self, size: u32) -> u32 {
        self.allocator.allocate(size)
    }

    /// Check if buffer needs to be resized
    pub fn needs_resize(&self) -> bool {
        self.allocator.total_size() > self.capacity
    }

    /// Get required capacity to fit all current allocations
    #[allow(dead_code)]
    pub fn required_capacity(&self) -> u32 {
        // Round up to next power of 2 for efficient growth
        let required = self.allocator.total_size();
        required.next_power_of_two()
    }
}

/// Manages all uniform buffers for the render system
pub struct UniformBufferManager {
    /// Shared buffer for all camera uniforms (view, proj matrices)
    pub camera_buffer: Option<DynamicUniformBuffer>,
    /// Shared buffer for all model uniforms (model matrices)
    pub model_buffer: Option<DynamicUniformBuffer>,
    /// Shared buffer for material uniforms (colors, parameters, etc.)
    #[allow(dead_code)]
    pub material_buffer: Option<DynamicUniformBuffer>,
    /// GPU device alignment requirement
    min_uniform_buffer_offset_alignment: u32,
}

impl UniformBufferManager {
    /// Create a new uniform buffer manager
    /// Buffers are created lazily when first allocation is needed
    pub fn new(min_uniform_buffer_offset_alignment: u32) -> Self {
        Self {
            camera_buffer: None,
            model_buffer: None,
            material_buffer: None,
            min_uniform_buffer_offset_alignment,
        }
    }

    /// Allocate space for camera uniforms
    /// Creates buffer on first call
    pub fn allocate_camera(&mut self, device: &wgpu::Device, size: u32) -> u32 {
        if self.camera_buffer.is_none() {
            // Initial capacity: 16 cameras * 256 bytes = 4KB
            self.camera_buffer = Some(DynamicUniformBuffer::new(
                device,
                self.min_uniform_buffer_offset_alignment,
                4096,
                "Camera Uniform Buffer",
            ));
        }

        let buffer = self.camera_buffer.as_mut().unwrap();
        let offset = buffer.allocate(size);

        // TODO: Handle buffer resize if needed
        if buffer.needs_resize() {
            log::warn!("Camera uniform buffer needs resize - not yet implemented");
        }

        offset
    }

    /// Allocate space for model uniforms
    pub fn allocate_model(&mut self, device: &wgpu::Device, size: u32) -> u32 {
        if self.model_buffer.is_none() {
            // Initial capacity: 64 models * 256 bytes = 16KB
            self.model_buffer = Some(DynamicUniformBuffer::new(
                device,
                self.min_uniform_buffer_offset_alignment,
                16384,
                "Model Uniform Buffer",
            ));
        }

        let buffer = self.model_buffer.as_mut().unwrap();
        let offset = buffer.allocate(size);

        if buffer.needs_resize() {
            log::warn!("Model uniform buffer needs resize - not yet implemented");
        }

        offset
    }

    /// Allocate space for material uniforms
    #[allow(dead_code)]
    pub fn allocate_material(&mut self, device: &wgpu::Device, size: u32) -> u32 {
        if self.material_buffer.is_none() {
            // Initial capacity: 32 materials * 256 bytes = 8KB
            self.material_buffer = Some(DynamicUniformBuffer::new(
                device,
                self.min_uniform_buffer_offset_alignment,
                8192,
                "Material Uniform Buffer",
            ));
        }

        let buffer = self.material_buffer.as_mut().unwrap();
        let offset = buffer.allocate(size);

        if buffer.needs_resize() {
            log::warn!("Material uniform buffer needs resize - not yet implemented");
        }

        offset
    }

    /// Write camera data to buffer at given offset
    pub fn write_camera_data(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        if let Some(buffer) = &self.camera_buffer {
            queue.write_buffer(&buffer.buffer, offset as u64, data);
        }
    }

    /// Write model data to buffer at given offset
    pub fn write_model_data(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        if let Some(buffer) = &self.model_buffer {
            queue.write_buffer(&buffer.buffer, offset as u64, data);
        }
    }

    /// Write material data to buffer at given offset
    #[allow(dead_code)]
    pub fn write_material_data(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        if let Some(buffer) = &self.material_buffer {
            queue.write_buffer(&buffer.buffer, offset as u64, data);
        }
    }
}
