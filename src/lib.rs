mod core;

// ============================================================================
// WASM Exports - for browser WASM via wasm-bindgen
// ============================================================================
#[cfg(feature = "wasm")]
mod wasm_exports {
    use super::core;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct BufferResult {
        buffer: Vec<u8>,
        result: u32,
    }

    #[wasm_bindgen]
    impl BufferResult {
        #[wasm_bindgen(getter)]
        pub fn buffer(&self) -> Vec<u8> {
            self.buffer.clone()
        }

        #[wasm_bindgen(getter)]
        pub fn result(&self) -> u32 {
            self.result
        }
    }

    #[wasm_bindgen]
    pub fn vulfram_init() -> u32 {
        core::vulfram_init() as u32
    }

    #[wasm_bindgen]
    pub fn vulfram_dispose() -> u32 {
        core::vulfram_dispose() as u32
    }

    #[wasm_bindgen]
    pub fn vulfram_send_queue(data: &[u8]) -> u32 {
        core::vulfram_send_queue(data.as_ptr(), data.len()) as u32
    }

    #[wasm_bindgen]
    pub fn vulfram_receive_queue() -> BufferResult {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_queue(ptr_ptr, length_ptr) as u32;
        if result != 0 || length == 0 {
            return BufferResult {
                buffer: Vec::new(),
                result,
            };
        }

        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        BufferResult {
            buffer: boxed.into_vec(),
            result,
        }
    }

    #[wasm_bindgen]
    pub fn vulfram_receive_events() -> BufferResult {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_events(ptr_ptr, length_ptr) as u32;
        if result != 0 || length == 0 {
            return BufferResult {
                buffer: Vec::new(),
                result,
            };
        }

        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        BufferResult {
            buffer: boxed.into_vec(),
            result,
        }
    }

    #[wasm_bindgen]
    pub fn vulfram_upload_buffer(id: u64, upload_type: u32, data: &[u8]) -> u32 {
        core::vulfram_upload_buffer(id, upload_type, data.as_ptr(), data.len()) as u32
    }

    #[wasm_bindgen]
    pub fn vulfram_tick(time_ms: f64, delta_ms: u32) -> u32 {
        core::vulfram_tick(time_ms as u64, delta_ms) as u32
    }

    #[wasm_bindgen]
    pub fn vulfram_get_profiling() -> BufferResult {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_get_profiling(ptr_ptr, length_ptr) as u32;
        if result != 0 || length == 0 {
            return BufferResult {
                buffer: Vec::new(),
                result,
            };
        }

        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        BufferResult {
            buffer: boxed.into_vec(),
            result,
        }
    }
}

// ============================================================================
// N-API Exports - for Node.js native modules
// ============================================================================
#[cfg(feature = "napi")]
#[allow(unused)]
mod napi_exports {
    use super::core;
    use napi::bindgen_prelude::*;
    use napi_derive::napi;

    #[napi(object)]
    pub struct BufferResult {
        pub buffer: Buffer,
        pub result: u32,
    }

    #[napi]
    pub fn vulfram_init() -> u32 {
        core::vulfram_init() as u32
    }

    #[napi]
    pub fn vulfram_dispose() -> u32 {
        core::vulfram_dispose() as u32
    }

    #[napi]
    pub fn vulfram_send_queue(data: Buffer) -> u32 {
        let ptr = data.as_ptr();
        let length = data.len();
        core::vulfram_send_queue(ptr, length) as u32
    }

    #[napi]
    pub fn vulfram_receive_queue() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Reconstruct Box<[u8]> and convert to Vec (zero-copy)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let vec = boxed.into_vec();
        let buffer = Buffer::from(vec);

        Ok(BufferResult { buffer, result })
    }

    #[napi]
    pub fn vulfram_receive_events() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Reconstruct Box<[u8]> and convert to Vec (zero-copy)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let vec = boxed.into_vec();
        let buffer = Buffer::from(vec);

        Ok(BufferResult { buffer, result })
    }

    #[napi]
    pub fn vulfram_upload_buffer(id: i64, upload_type: u32, data: Buffer) -> u32 {
        let ptr = data.as_ptr();
        let length = data.len();
        core::vulfram_upload_buffer(id as u64, upload_type, ptr, length) as u32
    }

    #[napi]
    pub fn vulfram_tick(time: i64, delta_time: u32) -> u32 {
        core::vulfram_tick(time as u64, delta_time) as u32
    }

    #[napi]
    pub fn vulfram_get_profiling() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Reconstruct Box<[u8]> and convert to Vec (zero-copy)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let vec = boxed.into_vec();
        let buffer = Buffer::from(vec);

        Ok(BufferResult { buffer, result })
    }
}

// ============================================================================
// Lua Exports - for Lua bindings via mlua
// ============================================================================
#[cfg(feature = "lua")]
#[allow(unused)]
mod lua_exports {
    use super::core;
    use mlua::prelude::*;

    fn vulfram_init(_: &Lua, _: ()) -> LuaResult<u32> {
        Ok(core::vulfram_init() as u32)
    }

    fn vulfram_dispose(_: &Lua, _: ()) -> LuaResult<u32> {
        Ok(core::vulfram_dispose() as u32)
    }

    fn vulfram_send_queue(_: &Lua, data: LuaString) -> LuaResult<u32> {
        let bytes = data.as_bytes();
        Ok(core::vulfram_send_queue(bytes.as_ptr(), bytes.len()) as u32)
    }

    fn vulfram_receive_queue(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        // Reconstruct Box<[u8]> and let Lua copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let lua_string = lua.create_string(&boxed)?;

        Ok((lua_string, result))
    }

    fn vulfram_receive_events(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        // Reconstruct Box<[u8]> and let Lua copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let lua_string = lua.create_string(&boxed)?;

        Ok((lua_string, result))
    }

    fn vulfram_upload_buffer(
        _: &Lua,
        (id, upload_type, data): (i64, u32, LuaString),
    ) -> LuaResult<u32> {
        let bytes = data.as_bytes();
        Ok(core::vulfram_upload_buffer(id as u64, upload_type, bytes.as_ptr(), bytes.len()) as u32)
    }

    fn vulfram_tick(_: &Lua, (time, delta_time): (i64, u32)) -> LuaResult<u32> {
        Ok(core::vulfram_tick(time as u64, delta_time) as u32)
    }

    fn vulfram_get_profiling(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        // Reconstruct Box<[u8]> and let Lua copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let lua_string = lua.create_string(&boxed)?;

        Ok((lua_string, result))
    }

    #[mlua::lua_module]
    pub fn vulfram(lua: &Lua) -> LuaResult<LuaTable> {
        let exports = lua.create_table()?;
        exports.set("init", lua.create_function(vulfram_init)?)?;
        exports.set("dispose", lua.create_function(vulfram_dispose)?)?;
        exports.set("send_queue", lua.create_function(vulfram_send_queue)?)?;
        exports.set("receive_queue", lua.create_function(vulfram_receive_queue)?)?;
        exports.set(
            "receive_events",
            lua.create_function(vulfram_receive_events)?,
        )?;
        exports.set("upload_buffer", lua.create_function(vulfram_upload_buffer)?)?;
        exports.set("tick", lua.create_function(vulfram_tick)?)?;
        exports.set("get_profiling", lua.create_function(vulfram_get_profiling)?)?;
        Ok(exports)
    }
}

// ============================================================================
// Python Exports - for Python bindings via PyO3
// ============================================================================
#[cfg(feature = "python")]
#[allow(unused)]
mod python_exports {
    use super::core;
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    #[pyfunction]
    fn vulfram_init() -> u32 {
        core::vulfram_init() as u32
    }

    #[pyfunction]
    fn vulfram_dispose() -> u32 {
        core::vulfram_dispose() as u32
    }

    #[pyfunction]
    fn vulfram_send_queue(data: &[u8]) -> u32 {
        core::vulfram_send_queue(data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn vulfram_receive_queue(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pyfunction]
    fn vulfram_receive_events(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pyfunction]
    fn vulfram_upload_buffer(id: i64, upload_type: u32, data: &[u8]) -> u32 {
        core::vulfram_upload_buffer(id as u64, upload_type, data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn vulfram_tick(time: i64, delta_time: u32) -> u32 {
        core::vulfram_tick(time as u64, delta_time) as u32
    }

    #[pyfunction]
    fn vulfram_get_profiling(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::vulfram_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Reconstruct Box<[u8]> and let Python copy (unavoidable)
        let boxed =
            unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, length)) };
        let py_bytes = PyBytes::new(py, &boxed).into();

        Ok((py_bytes, result))
    }

    #[pymodule]
    fn vulfram(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(vulfram_init, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_dispose, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_send_queue, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_receive_queue, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_receive_events, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_upload_buffer, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_tick, module)?)?;
        module.add_function(wrap_pyfunction!(vulfram_get_profiling, module)?)?;
        Ok(())
    }
}

// ============================================================================
// FFI (C ABI) Exports - for Bun/Node FFI via dlopen
// ============================================================================
#[cfg(feature = "ffi")]
#[allow(unused)]
mod ffi_exports {
    use super::core;

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_init() -> u32 {
        core::vulfram_init() as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_dispose() -> u32 {
        core::vulfram_dispose() as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_send_queue(ptr: *const u8, length: usize) -> u32 {
        core::vulfram_send_queue(ptr, length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_receive_queue(
        out_ptr: *mut *const u8,
        out_length: *mut usize,
    ) -> u32 {
        core::vulfram_receive_queue(out_ptr, out_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_receive_events(
        out_ptr: *mut *const u8,
        out_length: *mut usize,
    ) -> u32 {
        core::vulfram_receive_events(out_ptr, out_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_upload_buffer(
        bfr_id: u64,
        upload_type: u32,
        bfr_ptr: *const u8,
        bfr_length: usize,
    ) -> u32 {
        core::vulfram_upload_buffer(bfr_id, upload_type, bfr_ptr, bfr_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_tick(time: u64, delta_time: u32) -> u32 {
        core::vulfram_tick(time, delta_time) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn vulfram_get_profiling(
        out_ptr: *mut *const u8,
        out_length: *mut usize,
    ) -> u32 {
        core::vulfram_get_profiling(out_ptr, out_length) as u32
    }
}
