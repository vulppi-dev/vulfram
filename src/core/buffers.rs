use super::result::EngineResult;
use super::singleton::with_engine;

/// Upload data to a buffer identified by ID
pub fn engine_upload_buffer(bfr_id: u64, bfr_ptr: *const u8, bfr_length: usize) -> EngineResult {
    let data = unsafe { std::slice::from_raw_parts(bfr_ptr, bfr_length).to_vec() };

    match with_engine(|engine| {
        engine.buffers.insert(bfr_id, data);
    }) {
        Err(e) => e,
        Ok(_) => EngineResult::Success,
    }
}

/// Download data from a buffer identified by ID
pub fn engine_download_buffer(
    bfr_id: u64,
    bfr_ptr: *mut u8,
    bfr_length: *mut usize,
) -> EngineResult {
    match with_engine(|engine| {
        let buffer = match engine.buffers.get(&bfr_id) {
            Some(buf) => buf,
            None => return EngineResult::UnknownError,
        };

        let required_length = buffer.len();

        unsafe {
            if bfr_ptr.is_null() {
                *bfr_length = required_length;
                return EngineResult::Success;
            }

            let available_length = *bfr_length;

            if required_length <= available_length {
                std::ptr::copy_nonoverlapping(buffer.as_ptr(), bfr_ptr, required_length);
                *bfr_length = required_length;
                return EngineResult::Success;
            } else {
                *bfr_length = required_length;
                return EngineResult::BufferOverflow;
            }
        }
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

/// Clear a buffer identified by ID
pub fn engine_clear_buffer(bfr_id: u64) -> EngineResult {
    match with_engine(|engine| {
        engine.buffers.remove(&bfr_id);
    }) {
        Err(e) => return e,
        Ok(_) => EngineResult::Success,
    }
}
