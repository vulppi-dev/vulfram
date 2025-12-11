use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// MARK: - Uniform Types

/// All possible uniform types in WGSL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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

    /// Inject automatic uniform values (camera, model transforms)
    pub fn inject_automatic_uniforms(
        &self,
        buffer_data: &mut [u8],
        time: Option<f32>,
        delta_time: Option<f32>,
        camera_view: Option<&glam::Mat4>,
        camera_projection: Option<&glam::Mat4>,
        camera_view_projection: Option<&glam::Mat4>,
        camera_position: Option<&glam::Vec3>,
        model_transform: Option<&glam::Mat4>,
        model_normal: Option<&glam::Mat3>,
    ) {
        for field in &self.fields {
            // Extract values before match to ensure proper lifetimes
            let time_val = time.unwrap_or(0.0);
            let delta_time_val = delta_time.unwrap_or(0.0);

            let (auto_value, size) = match field.name.as_str() {
                "time" if time.is_some() => (bytemuck::bytes_of(&time_val), 4),
                "delta_time" if delta_time.is_some() => (bytemuck::bytes_of(&delta_time_val), 4),
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
                "model_normal" if model_normal.is_some() => {
                    (bytemuck::bytes_of(model_normal.unwrap()), 48)
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

// Removed: UniformBufferAllocator, DynamicUniformBuffer, UniformBufferManager
// These abstractions were not needed. The system uses ShaderUniformBuffers + BufferAllocator directly.
