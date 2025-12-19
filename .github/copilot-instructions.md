# Copilot Instructions — vulfram-core (Rust)

These instructions exist to keep Copilot suggestions consistent with the current codebase and its ABI/binding contracts.

## Language & style

- Human communication: pt-BR.
- Code identifiers: English (functions, variables, types).
- Formatting: rustfmt (VSCode + rust-analyzer formats on save). Do not add “manual formatting rules”.

## Scope / non-goals

- This repository is **Rust-only**. Do **not** generate or maintain TypeScript/JavaScript here.
- Do **not** introduce or export internal engine/core Rust types. Only the small `vulfram_*` API surface is exported.

## Crate profile (Cargo.toml)

- `edition = "2024"`
- `crate-type = ["cdylib"]`
- Features:
  - `default = ["napi"]`
  - optional: `ffi`, `lua`, `python`

## Runtime bindings implemented in Rust

Bindings are implemented behind feature flags:

- **N-API (default)**: Node.js native module via `napi` / `napi-derive`.
- **Lua (optional)**: via `mlua`.
- **Python (optional)**: via `pyo3`.
- **Raw C ABI (optional)**: `extern "C"` exports behind `feature = "ffi"`.

Keep the binding modules gated by `#[cfg(feature = "...")]` and avoid cross-feature symbol drift.

## Exported API surface

### Status codes

- All `vulfram_*` entrypoints return a `u32` status code (`0 = success`, non-zero = error).

### Lifecycle

- `vulfram_init() -> u32`
- `vulfram_dispose() -> u32`
- `vulfram_tick(time, delta_time) -> u32`
  - `time`: `u64` in raw C ABI; `i64` in some bindings (casted).

### Command queue (Host → Core)

- `vulfram_send_queue(...) -> u32`  
  Receives a byte buffer with a **MessagePack** batch (via `serde` + `rmp-serde`).

### Output queues (Core → Host)

- `vulfram_receive_queue(...) -> (buffer, status)`
- `vulfram_receive_events(...) -> (buffer, status)`
- `vulfram_get_profiling(...) -> (buffer, status)` **(note name: `get_profiling`)**

All returned buffers are **opaque byte blobs** (usually MessagePack). The meaning/format is defined by the host protocol, not by exported Rust types.

### Upload / download byte buffers

- `vulfram_upload_buffer(id, upload_type, data) -> u32`
  - `id`: host-generated logical ID
  - `upload_type`: `u32` discriminator
  - `data`: byte buffer
- `vulfram_download_buffer(id) -> (buffer, status)` (bindings)  
  Raw C ABI may use out-pointers for `(ptr, len)`.

## Buffer ownership & zero-copy rules (CRITICAL)

The core returns outbound data as a `(ptr, len)` pair via out-params. The **binding layer** must take ownership exactly once and ensure the allocation is eventually freed.

### N-API (zero-copy into JS Buffer)

- Convert `(ptr, len)` to `Box<[u8]>` (from_raw on a `slice::from_raw_parts_mut`), then to `Vec<u8>`, then to `napi::Buffer::from(vec)`.
- This is **zero-copy**: the bytes are moved into the JavaScript engine’s `Buffer`.
- After `Buffer::from(vec)`, the **JS GC owns the memory** and will free it. Do not add manual free logic.

### Lua / Python (copy unavoidable)

- Convert `(ptr, len)` back to `Box<[u8]>`, then create `LuaString` / `PyBytes` from it.
- These runtimes **copy** the bytes.
- The `Box<[u8]>` must be dropped immediately after the copy.

### Raw C ABI (feature = "ffi")

- There is no runtime wrapper to automatically reclaim allocations.
- If you extend or use this path, keep ownership explicit:
  - Prefer adding an explicit `vulfram_free_buffer(ptr, len)` export **or**
  - Change the API to require caller-allocated buffers (no owned pointer returned).
- Do **not** “leak by design” (e.g., `mem::forget`) without a matching reclamation mechanism.

### Safety invariants

- The `(ptr, len)` returned by core must reference a Rust heap allocation that is valid for exactly one ownership transfer.
- Never reconstruct the same allocation twice.
- Never assume the pointer is valid after transferring ownership to `Buffer::from(vec)`.

## Concurrency & threading

- Assume `vulfram_*` calls are main-thread-only.
- Do not introduce shared mutable global state without synchronization. Keep thread-safety decisions explicit.

## Coding conventions

- Prefer self-explanatory names over commentary.
- Avoid redundant comments. Only comment when needed for:
  - `unsafe` blocks and FFI invariants,
  - tricky lifetime/ownership behavior,
  - non-obvious algorithms or performance tradeoffs.
- Keep exported interfaces stable; do not add new exported entrypoints unless explicitly requested.

## Key dependencies (for correct APIs)

- `wgpu = "27.0"`
- `winit = "0.30"`
- `gilrs = "0.11"`
- `serde = "1.0"`, `rmp-serde = "1.3.0"`, `serde_repr = "0.1"`
- Optional bindings:
  - `napi = "3.6"`, `napi-derive = "3.4"`
  - `mlua = "0.11.5"`
  - `pyo3 = "0.27.2"`
