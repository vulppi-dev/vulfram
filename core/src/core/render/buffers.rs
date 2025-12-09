use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use wgpu;

// MARK: - Uniform Types

/// All possible uniform types in WGSL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum UniformType {
    // Scalars
    Float = 0,
    Int,
    UInt,
    Bool,

    // Vectors
    Vec2,
    Vec3,
    Vec4,
    Vec2i,
    Vec3i,
    Vec4i,
    Vec2u,
    Vec3u,
    Vec4u,

    // Matrices (column-major)
    Mat2x2,
    Mat2x3,
    Mat2x4,
    Mat3x2,
    Mat3x3,
    Mat3x4,
    Mat4x2,
    Mat4x3,
    Mat4x4,

    // Atomic types (for storage buffers)
    AtomicInt,
    AtomicUInt,
}

impl UniformType {
    /// Returns (size, alignment) following std140 layout rules
    pub fn size_and_alignment(&self) -> (u32, u32) {
        match self {
            // Scalars
            UniformType::Float => (4, 4),
            UniformType::Int => (4, 4),
            UniformType::UInt => (4, 4),
            UniformType::Bool => (4, 4), // bool is treated as u32

            // Vectors
            UniformType::Vec2 => (8, 8),
            UniformType::Vec3 => (12, 16), // Vec3 aligns as Vec4!
            UniformType::Vec4 => (16, 16),
            UniformType::Vec2i => (8, 8),
            UniformType::Vec3i => (12, 16),
            UniformType::Vec4i => (16, 16),
            UniformType::Vec2u => (8, 8),
            UniformType::Vec3u => (12, 16),
            UniformType::Vec4u => (16, 16),

            // Matrices (column-major, each column is a vec)
            UniformType::Mat2x2 => (16, 8),  // 2 vec2
            UniformType::Mat2x3 => (32, 16), // 2 vec3 (padded to vec4)
            UniformType::Mat2x4 => (32, 16), // 2 vec4
            UniformType::Mat3x2 => (24, 8),  // 3 vec2
            UniformType::Mat3x3 => (48, 16), // 3 vec3 (padded to vec4)
            UniformType::Mat3x4 => (48, 16), // 3 vec4
            UniformType::Mat4x2 => (32, 8),  // 4 vec2
            UniformType::Mat4x3 => (64, 16), // 4 vec3 (padded to vec4)
            UniformType::Mat4x4 => (64, 16), // 4 vec4

            // Atomics (storage only)
            UniformType::AtomicInt => (4, 4),
            UniformType::AtomicUInt => (4, 4),
        }
    }
}

/// Uniform values using glam types (with bytemuck support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum UniformValue {
    // Scalars
    Float(f32),
    Int(i32),
    UInt(u32),
    Bool(bool),

    // Vectors (glam types with bytemuck already implemented)
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    Vec2i(glam::IVec2),
    Vec3i(glam::IVec3),
    Vec4i(glam::IVec4),
    Vec2u(glam::UVec2),
    Vec3u(glam::UVec3),
    Vec4u(glam::UVec4),

    // Matrices (glam types with bytemuck already implemented)
    Mat2(glam::Mat2),
    Mat3(glam::Mat3),
    Mat4(glam::Mat4),

    // Arrays for special cases
    FloatArray(Vec<f32>),
    Vec4Array(Vec<glam::Vec4>),
    Mat4Array(Vec<glam::Mat4>),
}

impl UniformValue {
    /// Get bytes representation using bytemuck
    /// Returns a Vec to avoid lifetime issues with bool conversion
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            UniformValue::Float(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Int(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::UInt(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Bool(v) => {
                let as_u32 = *v as u32;
                bytemuck::bytes_of(&as_u32).to_vec()
            }

            // glam types - bytemuck::Pod already implemented
            UniformValue::Vec2(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec3(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec4(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec2i(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec3i(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec4i(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec2u(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec3u(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Vec4u(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Mat2(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Mat3(v) => bytemuck::bytes_of(v).to_vec(),
            UniformValue::Mat4(v) => bytemuck::bytes_of(v).to_vec(),

            // Arrays - cast_slice direct
            UniformValue::FloatArray(v) => bytemuck::cast_slice(v).to_vec(),
            UniformValue::Vec4Array(v) => bytemuck::cast_slice(v).to_vec(),
            UniformValue::Mat4Array(v) => bytemuck::cast_slice(v).to_vec(),
        }
    }

    /// Check if value matches expected uniform type
    pub fn matches_type(&self, uniform_type: UniformType) -> bool {
        matches!(
            (self, uniform_type),
            (UniformValue::Float(_), UniformType::Float)
                | (UniformValue::Int(_), UniformType::Int)
                | (UniformValue::UInt(_), UniformType::UInt)
                | (UniformValue::Bool(_), UniformType::Bool)
                | (UniformValue::Vec2(_), UniformType::Vec2)
                | (UniformValue::Vec3(_), UniformType::Vec3)
                | (UniformValue::Vec4(_), UniformType::Vec4)
                | (UniformValue::Vec2i(_), UniformType::Vec2i)
                | (UniformValue::Vec3i(_), UniformType::Vec3i)
                | (UniformValue::Vec4i(_), UniformType::Vec4i)
                | (UniformValue::Vec2u(_), UniformType::Vec2u)
                | (UniformValue::Vec3u(_), UniformType::Vec3u)
                | (UniformValue::Vec4u(_), UniformType::Vec4u)
                | (UniformValue::Mat2(_), UniformType::Mat2x2)
                | (UniformValue::Mat3(_), UniformType::Mat3x3)
                | (UniformValue::Mat4(_), UniformType::Mat4x4)
        )
    }
}

// MARK: - Uniform Layout

/// Field definition within a uniform buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: UniformType,
}

/// Calculated layout for a uniform field (with offset)
#[derive(Debug, Clone)]
pub struct UniformFieldLayout {
    pub name: String,
    pub field_type: UniformType,
    pub offset: u32,
    pub size: u32,
}

/// Layout of a complete uniform buffer with calculated offsets
#[derive(Debug, Clone)]
pub struct UniformBufferLayout {
    pub group: u32,
    pub binding: u32,
    pub fields: Vec<UniformFieldLayout>,
    pub total_size: u32,
}

impl UniformBufferLayout {
    /// Calculate layout from field definitions
    pub fn from_fields(group: u32, binding: u32, fields: &[UniformField]) -> Self {
        let mut current_offset = 0u32;
        let mut field_layouts = Vec::with_capacity(fields.len());

        for field in fields {
            let (size, alignment) = field.field_type.size_and_alignment();

            // Align offset
            current_offset = align_to(current_offset, alignment);

            field_layouts.push(UniformFieldLayout {
                name: field.name.clone(), // Necessário - ownership transfer
                field_type: field.field_type,
                offset: current_offset,
                size,
            });

            current_offset += size;
        }

        // Align total size to 16 bytes (WGSL requirement)
        let total_size = align_to(current_offset, 16);

        Self {
            group,
            binding,
            fields: field_layouts,
            total_size,
        }
    }

    /// Pack uniform values into byte buffer following this layout
    pub fn pack_values(&self, values: &HashMap<String, UniformValue>) -> Result<Vec<u8>, String> {
        let mut packed = vec![0u8; self.total_size as usize];

        for field in &self.fields {
            let value = values
                .get(&field.name)
                .ok_or_else(|| format!("Missing uniform value: {}", field.name))?;

            if !value.matches_type(field.field_type) {
                return Err(format!(
                    "Type mismatch for '{}': expected {:?}",
                    field.name, field.field_type
                ));
            }

            // Copy bytes directly
            let bytes = value.as_bytes();
            let offset = field.offset as usize;
            let end = offset + bytes.len();
            if end > packed.len() {
                return Err(format!(
                    "Buffer overflow for field '{}': offset {} + size {} > total size {}",
                    field.name,
                    offset,
                    bytes.len(),
                    packed.len()
                ));
            }
            packed[offset..end].copy_from_slice(&bytes);
        }

        Ok(packed)
    }

    /// Inject automatic uniform values (camera, model transforms)
    pub fn inject_automatic_uniforms(
        &self,
        buffer_data: &mut [u8],
        camera_view: Option<&glam::Mat4>,
        camera_projection: Option<&glam::Mat4>,
        camera_view_projection: Option<&glam::Mat4>,
        camera_position: Option<&glam::Vec3>,
        model_transform: Option<&glam::Mat4>,
        model_normal_matrix: Option<&glam::Mat3>,
    ) {
        for field in &self.fields {
            let (auto_value, size) = match field.name.as_str() {
                "camera_view" if camera_view.is_some() => {
                    (bytemuck::bytes_of(camera_view.unwrap()), 64)
                }
                "camera_projection" if camera_projection.is_some() => {
                    (bytemuck::bytes_of(camera_projection.unwrap()), 64)
                }
                "camera_view_projection" if camera_view_projection.is_some() => {
                    (bytemuck::bytes_of(camera_view_projection.unwrap()), 64)
                }
                "camera_position" if camera_position.is_some() => {
                    (bytemuck::bytes_of(camera_position.unwrap()), 12)
                }
                "model_transform" if model_transform.is_some() => {
                    (bytemuck::bytes_of(model_transform.unwrap()), 64)
                }
                "model_normal_matrix" if model_normal_matrix.is_some() => {
                    (bytemuck::bytes_of(model_normal_matrix.unwrap()), 48)
                }
                _ => continue,
            };

            let offset = field.offset as usize;
            buffer_data[offset..offset + size].copy_from_slice(auto_value);
        }
    }
}

/// Align value to specified alignment
fn align_to(value: u32, alignment: u32) -> u32 {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

// MARK: - Buffer Allocator

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
    /// Optional layout information for validation
    pub layout: Option<UniformBufferLayout>,
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
            layout: None,
        }
    }

    /// Create a new dynamic uniform buffer with a specific layout
    pub fn with_layout(
        device: &wgpu::Device,
        alignment: u32,
        initial_capacity: u32,
        label: &str,
        layout: UniformBufferLayout,
    ) -> Self {
        let mut buffer = Self::new(device, alignment, initial_capacity, label);
        buffer.layout = Some(layout);
        buffer
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
    pub fn required_capacity(&self) -> u32 {
        // Round up to next power of 2 for efficient growth
        let required = self.allocator.total_size();
        required.next_power_of_two()
    }

    /// Write uniform data following the layout (if available)
    pub fn write_data(
        &self,
        queue: &wgpu::Queue,
        offset: u32,
        values: &HashMap<String, UniformValue>,
    ) -> Result<(), String> {
        if let Some(layout) = &self.layout {
            let packed = layout.pack_values(values)?;
            queue.write_buffer(&self.buffer, offset as u64, &packed);
            Ok(())
        } else {
            Err("Buffer has no layout - cannot pack values".to_string())
        }
    }

    /// Write raw bytes directly
    pub fn write_bytes(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        queue.write_buffer(&self.buffer, offset as u64, data);
    }
}

// MARK: - Uniform Buffer Manager

/// Manages dynamic uniform buffers for shaders
/// Buffers can be created with specific layouts or used generically
pub struct UniformBufferManager {
    /// Shared buffers indexed by a key (shader_id, material_id, etc.)
    buffers: HashMap<String, DynamicUniformBuffer>,
    /// GPU device alignment requirement
    min_uniform_buffer_offset_alignment: u32,
}

impl UniformBufferManager {
    /// Create a new uniform buffer manager
    pub fn new(min_uniform_buffer_offset_alignment: u32) -> Self {
        Self {
            buffers: HashMap::new(),
            min_uniform_buffer_offset_alignment,
        }
    }

    /// Create or get a uniform buffer with a specific layout
    pub fn get_or_create_buffer(
        &mut self,
        device: &wgpu::Device,
        key: &str,
        initial_capacity: u32,
        layout: Option<UniformBufferLayout>,
    ) -> &mut DynamicUniformBuffer {
        // Use entry API to avoid double lookup and unnecessary to_string
        self.buffers.entry(key.to_owned()).or_insert_with(|| {
            if let Some(layout) = layout {
                DynamicUniformBuffer::with_layout(
                    device,
                    self.min_uniform_buffer_offset_alignment,
                    initial_capacity,
                    key,
                    layout,
                )
            } else {
                DynamicUniformBuffer::new(
                    device,
                    self.min_uniform_buffer_offset_alignment,
                    initial_capacity,
                    key,
                )
            }
        })
    }

    /// Get an existing buffer
    pub fn get_buffer(&self, key: &str) -> Option<&DynamicUniformBuffer> {
        self.buffers.get(key)
    }

    /// Get a mutable reference to an existing buffer
    pub fn get_buffer_mut(&mut self, key: &str) -> Option<&mut DynamicUniformBuffer> {
        self.buffers.get_mut(key)
    }

    /// Allocate space in a specific buffer
    pub fn allocate(&mut self, device: &wgpu::Device, key: &str, size: u32) -> Result<u32, String> {
        let buffer = self.get_or_create_buffer(device, key, size * 16, None);

        let offset = buffer.allocate(size);

        if buffer.needs_resize() {
            log::warn!("Buffer '{}' needs resize - not yet implemented", key);
        }

        Ok(offset)
    }

    /// Write data to a buffer at given offset
    pub fn write_data(
        &self,
        queue: &wgpu::Queue,
        key: &str,
        offset: u32,
        data: &[u8],
    ) -> Result<(), String> {
        let buffer = self
            .buffers
            .get(key)
            .ok_or_else(|| format!("Buffer '{}' not found", key))?;

        buffer.write_bytes(queue, offset, data);
        Ok(())
    }

    /// Write uniform values using buffer layout
    pub fn write_uniforms(
        &self,
        queue: &wgpu::Queue,
        key: &str,
        offset: u32,
        values: &HashMap<String, UniformValue>,
    ) -> Result<(), String> {
        let buffer = self
            .buffers
            .get(key)
            .ok_or_else(|| format!("Buffer '{}' not found", key))?;

        buffer.write_data(queue, offset, values)
    }

    /// Remove a buffer
    pub fn remove_buffer(&mut self, key: &str) {
        self.buffers.remove(key);
    }

    /// Clear all buffers
    pub fn clear(&mut self) {
        self.buffers.clear();
    }
}

// MARK: - Legacy compatibility helpers (deprecated)

impl UniformBufferManager {
    /// Allocate space for camera uniforms (legacy)
    #[deprecated(note = "Use get_or_create_buffer with proper layout instead")]
    pub fn allocate_camera(&mut self, device: &wgpu::Device, size: u32) -> u32 {
        self.allocate(device, "legacy_camera", size).unwrap_or(0)
    }

    /// Allocate space for model uniforms (legacy)
    #[deprecated(note = "Use get_or_create_buffer with proper layout instead")]
    pub fn allocate_model(&mut self, device: &wgpu::Device, size: u32) -> u32 {
        self.allocate(device, "legacy_model", size).unwrap_or(0)
    }

    /// Write camera data to buffer at given offset (legacy)
    #[deprecated(note = "Use write_data instead")]
    pub fn write_camera_data(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        let _ = self.write_data(queue, "legacy_camera", offset, data);
    }

    /// Write model data to buffer at given offset (legacy)
    #[deprecated(note = "Use write_data instead")]
    pub fn write_model_data(&self, queue: &wgpu::Queue, offset: u32, data: &[u8]) {
        let _ = self.write_data(queue, "legacy_model", offset, data);
    }
}
