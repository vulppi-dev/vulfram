use crate::core;
use crate::core::VulframResult;
use crate::core::buffers::state::UploadType;
use bytemuck::cast_slice;
use std::fs;

pub fn load_texture_bytes(path: &str) -> Vec<u8> {
    fs::read(path).expect("failed to read texture")
}

pub fn upload_texture_bytes(bytes: &[u8], buffer_id: u64) {
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(UploadType::ImageData),
            bytes.as_ptr(),
            bytes.len()
        ),
        VulframResult::Success
    );
}

pub fn upload_binary_bytes(bytes: &[u8], buffer_id: u64) {
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(UploadType::BinaryAsset),
            bytes.as_ptr(),
            bytes.len()
        ),
        VulframResult::Success
    );
}

pub fn upload_texture(path: &str, buffer_id: u64) {
    let texture_bytes = load_texture_bytes(path);
    upload_texture_bytes(&texture_bytes, buffer_id);
}

pub fn upload_buffer<T: bytemuck::Pod>(buffer_id: u64, upload_type: UploadType, data: &[T]) {
    let bytes = cast_slice(data);
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(upload_type),
            bytes.as_ptr() as *const u8,
            bytes.len()
        ),
        VulframResult::Success
    );
}

pub fn upload_type_to_u32(upload_type: UploadType) -> u32 {
    match upload_type {
        UploadType::Raw => 0,
        UploadType::ShaderSource => 1,
        UploadType::GeometryData => 2,
        UploadType::VertexData => 3,
        UploadType::IndexData => 4,
        UploadType::ImageData => 5,
        UploadType::BinaryAsset => 6,
    }
}
