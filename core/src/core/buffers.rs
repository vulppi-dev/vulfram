use super::result::VulframResult;
use super::singleton::with_engine;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Upload type - defines the purpose of the buffer data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum UploadType {
    /// Raw binary data (default)
    Raw = 0,
    /// Shader source code (WGSL, GLSL, SPIR-V)
    ShaderSource,
    /// Vertex data for geometry
    VertexData,
    /// Index data for geometry
    IndexData,
    /// Image data (PNG, JPEG, WebP, AVIF) - will be decoded when consumed
    ImageData,
    /// Generic binary asset
    BinaryAsset,
}

impl UploadType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(UploadType::Raw),
            1 => Some(UploadType::ShaderSource),
            2 => Some(UploadType::VertexData),
            3 => Some(UploadType::IndexData),
            4 => Some(UploadType::ImageData),
            5 => Some(UploadType::BinaryAsset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadBuffer {
    pub upload_type: UploadType,
    pub data: Vec<u8>,
}

pub fn vulfram_upload_buffer(
    bfr_id: u64,
    upload_type: u32,
    bfr_ptr: *const u8,
    bfr_length: usize,
) -> VulframResult {
    // Validate upload type
    let upload_type = match UploadType::from_u32(upload_type) {
        Some(t) => t,
        None => return VulframResult::InvalidUploadType,
    };

    let data = unsafe { std::slice::from_raw_parts(bfr_ptr, bfr_length).to_vec() };

    match with_engine(|engine| {
        // Check for ID collision (one-shot semantics)
        if engine.buffers.contains_key(&bfr_id) {
            return VulframResult::BufferIdCollision;
        }

        // Store as raw bytes with type metadata
        // Decoding will happen when the buffer is consumed by a command
        let buffer = UploadBuffer { upload_type, data };

        engine.buffers.insert(bfr_id, buffer);
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

pub fn vulfram_download_buffer(
    bfr_id: u64,
    bfr_ptr: *mut *const u8,
    bfr_length: *mut usize,
) -> VulframResult {
    match with_engine(|engine| {
        let buffer = match engine.buffers.remove(&bfr_id) {
            Some(buf) => buf,
            None => {
                // Buffer not found - set null pointer and length 0
                unsafe {
                    *bfr_ptr = std::ptr::null();
                    *bfr_length = 0;
                }
                return VulframResult::BufferNotFound;
            }
        };

        let data_length = buffer.data.len();

        // Transfer ownership via Box::into_raw (zero-copy)
        let boxed = buffer.data.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u8;

        unsafe {
            *bfr_ptr = ptr;
            *bfr_length = data_length;
        }

        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
