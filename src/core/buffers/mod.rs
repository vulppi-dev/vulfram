use crate::core::VulframResult;
use crate::core::buffers::state::{BufferStorage, UploadBuffer, UploadType};
use crate::core::singleton::with_engine;

pub mod state;

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
        let storage: &mut BufferStorage = &mut engine.buffers;

        // Check for ID collision (one-shot semantics)
        if !storage.insert_upload(bfr_id, UploadBuffer { upload_type, data }) {
            return VulframResult::BufferIdCollision;
        }

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
        let storage: &mut BufferStorage = &mut engine.buffers;

        let buffer = match storage.remove_upload(bfr_id) {
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
