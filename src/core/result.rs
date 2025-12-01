/// Result codes returned by engine functions
#[derive(Debug)]
#[repr(u32)]
pub enum EngineResult {
    Success = 0,
    UnknownError = 1,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    BufferOverflow,
    // Reserved error codes for Winit 1000-1999
    WinitError = 1000,
    // Reserved error codes for WGPU 2000-2999
    WgpuError = 2000,
    // Reserved error codes for Command Processing 3000-3999
    CmdInvalidCborError = 3000,
}
