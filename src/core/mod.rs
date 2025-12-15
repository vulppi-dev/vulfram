// Submodules
pub mod buffers;
pub mod gamepad;
mod handler;
pub mod input;
mod lifecycle;
pub mod profiling;
mod queue;
pub mod render;
mod singleton;
mod state;
pub mod system;
mod tick;
pub mod window;

// Public modules
pub mod cmd;
pub mod image;

// Re-exports for public API
#[allow(unused)]
pub use buffers::{vulfram_download_buffer, vulfram_upload_buffer};
#[allow(unused)]
pub use lifecycle::{vulfram_dispose, vulfram_init};
#[allow(unused)]
pub use profiling::vulfram_get_profiling;
#[allow(unused)]
pub use queue::{vulfram_receive_events, vulfram_receive_queue, vulfram_send_queue};
#[allow(unused)]
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
