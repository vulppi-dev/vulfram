use crate::core::VulframResult;
use crate::core::buffers::state::{BufferStorage, UploadBuffer, UploadType};
use crate::core::singleton::with_engine;

pub mod cmd;
pub mod state;

pub use cmd::*;
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
