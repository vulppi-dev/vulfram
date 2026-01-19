#[cfg(not(feature = "wasm"))]
mod native;
#[cfg(feature = "wasm")]
mod web;

#[cfg(not(feature = "wasm"))]
pub use native::*;
#[cfg(feature = "wasm")]
pub use web::*;
