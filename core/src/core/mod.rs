// Submodules
pub mod buffers;
mod cache;
mod gamepad;
mod handler;
mod lifecycle;
mod profiling;
mod queue;
pub mod render;
mod singleton;
mod state;
mod tick;

// Public modules
pub mod cmd;
pub mod image;

// Re-exports for public API
#[allow(unused)]
pub use buffers::{vulfram_download_buffer, vulfram_upload_buffer};
pub use lifecycle::{vulfram_dispose, vulfram_init};
#[allow(unused)]
pub use profiling::vulfram_get_profiling;
pub use queue::{vulfram_receive_events, vulfram_receive_queue, vulfram_send_queue};
pub use tick::vulfram_tick;

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(unused)]
pub enum VulframResult {
    Success = 0,
    UnknownError,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    CmdInvalidMessagePackError,
    BufferNotFound,
    BufferIdCollision,
    InvalidUploadType,
}
