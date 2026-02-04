pub mod audio;
pub mod buffers;
pub mod cmd;
pub mod gamepad;
pub mod image;
pub mod input;
mod lifecycle;
pub mod platform;
pub mod platforms;
pub mod profiling;
mod queue;
pub mod render;
pub mod resources;
mod singleton;
mod state;
pub mod system;
mod tick;
pub mod ui;
pub mod window;

// Re-exports for public API
#[allow(unused)]
pub use buffers::vulfram_upload_buffer;
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
    NotInBrowser,
    CmdInvalidMessagePackError,
    BufferNotFound,
    BufferIdCollision,
    InvalidUploadType,
}
