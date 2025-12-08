use super::image::{ImageBuffer, ImageDecoder};
use super::result::VulframResult;
use super::singleton::with_engine;

/// Buffer data types - either raw bytes or decoded image
#[derive(Debug, Clone)]
pub enum BufferData {
    Raw(Vec<u8>),
    Image(ImageBuffer),
}

/// Buffer wrapper with typed data
#[derive(Debug, Clone)]
pub struct Buffer {
    pub data: BufferData,
}

/// Upload data to a buffer identified by ID
/// Automatically detects and decodes images (PNG, JPEG, WebP, AVIF)
pub fn vulfram_upload_buffer(bfr_id: u64, bfr_ptr: *const u8, bfr_length: usize) -> VulframResult {
    let data = unsafe { std::slice::from_raw_parts(bfr_ptr, bfr_length).to_vec() };

    match with_engine(|engine| {
        // Try to decode as image
        let buffer = if let Some(image_buffer) = ImageDecoder::try_decode(&data) {
            // Successfully decoded as image
            Buffer {
                data: BufferData::Image(image_buffer),
            }
        } else {
            // Not an image or failed to decode - store as raw
            Buffer {
                data: BufferData::Raw(data),
            }
        };

        engine.buffers.insert(bfr_id, buffer);
    }) {
        Err(e) => e,
        Ok(_) => VulframResult::Success,
    }
}

/// Download data from a buffer identified by ID
pub fn vulfram_download_buffer(
    bfr_id: u64,
    bfr_ptr: *mut u8,
    bfr_length: *mut usize,
) -> VulframResult {
    match with_engine(|engine| {
        let buffer = match engine.buffers.get(&bfr_id) {
            Some(buf) => buf,
            None => return VulframResult::UnknownError,
        };

        // Get raw bytes from buffer
        let bytes = match &buffer.data {
            BufferData::Raw(data) => data.as_slice(),
            BufferData::Image(img) => img.data.as_slice(),
        };

        let required_length = bytes.len();

        unsafe {
            if bfr_ptr.is_null() {
                *bfr_length = required_length;
                return VulframResult::Success;
            }

            let available_length = *bfr_length;

            if required_length <= available_length {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), bfr_ptr, required_length);
                *bfr_length = required_length;
                return VulframResult::Success;
            } else {
                *bfr_length = required_length;
                return VulframResult::BufferOverflow;
            }
        }
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
