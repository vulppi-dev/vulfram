// Submodules
pub mod buffers;
mod cache;
mod handler;
mod lifecycle;
mod profiling;
mod queue;
pub mod render;
pub mod result;
mod singleton;
mod state;
mod tick;

// Public modules
pub mod cmd;
pub mod image;

// Re-exports for public API
pub use buffers::{vulfram_download_buffer, vulfram_upload_buffer};
pub use lifecycle::{vulfram_dispose, vulfram_init};
pub use profiling::vulfram_get_profiling;
pub use queue::{vulfram_receive_events, vulfram_receive_queue, vulfram_send_queue};
pub use tick::vulfram_tick;
