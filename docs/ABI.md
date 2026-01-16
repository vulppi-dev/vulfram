# 🦊 Vulfram — Functions, ABI and Usage Contract

This document describes the **public C-ABI surface** exposed by the Vulfram core,
and the **usage contract** expected from hosts and language bindings.

It is intended for:

- Binding authors (N-API, `mlua`, `PyO3`, etc.)
- Advanced users who need to understand the low-level interaction.

---

## 1. Global Conventions

### 1.1 Return Type: `VulframResult (u32)`

All `vulfram_*` functions return a `u32` status code:

- `0` → success (`VULFRAM_SUCCESS`)
- non-zero → error (`VulframResult` enum)

The binding **must** map this to language-level error handling. For example:

- N-API: throw JS Error on non-zero
- Python: raise Python exception
- Lua: return `nil, err` or throw error, depending on style

The exact enum values live in the Rust core, but the important contract is:

- `0` = OK
- Non-zero = failure (with possible subcodes for detailed errors)

### 1.2 Threading / Reentrancy

- All `vulfram_*` functions are **main-thread only**.
- The host must **not** call them concurrently from multiple threads.
- Any multithreading should be implemented on the host side, with a
  single thread delegating to Vulfram.

### 1.3 Serialization Format

All structured data that crosses the ABI uses **MessagePack**, via:

- `serde`
- `rmp-serde`
- `serde_repr` for numeric enums

This includes:

- Command batches (`vulfram_send_queue`)
- Response/message batches (`vulfram_receive_queue`)
- Event batches (`vulfram_receive_events`)
- Profiling data (`vulfram_profiling`)

Binding responsibilities:

- Serialize commands into MessagePack when calling `send_queue`.
- Deserialize MessagePack returned by `receive_queue`, `receive_events`, `profiling`.

### 1.4 Output Buffers (`out_ptr`, `out_length`)

Several functions return data via pointer-out parameters:

```c
u32 vulfram_xxx(uint8_t** out_ptr, size_t* out_length);
```

Contract:

- Vulfram allocates a contiguous byte buffer and sets:

  - `*out_ptr` to the buffer address
  - `*out_length` to the number of bytes

- The **caller on the C side** (the binding) is responsible for:

  1. Copying the bytes into its own memory representation
     (JS Buffer, Python bytes, Lua string, etc.)
  2. Releasing the buffer using the deallocation mechanism defined in
     the dynamic library / binding.

The game code written in JS/Lua/Python **never** handles raw pointers.
The binding hides this complexity.

---

## 2. Function List

All functions have the `vulfram_` prefix and use C-ABI (`extern "C"`).

### 2.1 Initialization and Shutdown

```c
u32 vulfram_init(void);
u32 vulfram_dispose(void);
```

- `vulfram_init()`

  - Initializes the core state, subsystems, and any global allocations.
  - Must be called **exactly once** before any other function.

- `vulfram_dispose()`

  - Shuts down the core, frees resources, and tears down subsystems.
  - After calling this, no other `vulfram_*` functions may be used
    unless a fresh `vulfram_init()` is called (full restart).

---

### 2.2 Command Queue (Host → Core)

```c
u32 vulfram_send_queue(const uint8_t* buffer, size_t length);
```

- `buffer`
  Pointer to a MessagePack-encoded batch of **commands**.

- `length`
  Size of the buffer in bytes.

Core behavior:

1. Copies the `buffer` contents into internal memory.
2. Decodes it into one or more internal command structures.
3. Enqueues these commands to be processed on the next `vulfram_tick()`.

Typical commands include:

- Create/update/destroy **resources** (shaders, textures, geometries, materials…)
- Create/update/destroy **components** (cameras, models, etc.)
- Maintenance commands (e.g. discard unused uploads).

---

### 2.3 Response / Message Queue (Core → Host)

```c
u32 vulfram_receive_queue(uint8_t** out_ptr, size_t* out_length);
```

- On success, returns a MessagePack buffer with a batch of **messages**:

  - Acknowledgments, detailed error info, internal notifications, etc.

- The buffer may be empty:

  - `*out_length == 0` indicates “no messages available”.

Binding responsibilities:

1. Call `vulfram_receive_queue(&ptr, &len)`.
2. If `len > 0`, copy `[ptr .. ptr+len)` to host memory.
3. Release the core-allocated buffer using the agreed mechanism.
4. Deserialize MessagePack and route messages to the host/application.

Calling `vulfram_receive_queue` consumes and clears the internal response queue.

---

### 2.4 Event Queue (Input / Window)

```c
u32 vulfram_receive_events(uint8_t** out_ptr, size_t* out_length);
```

- Returns a MessagePack buffer containing a batch of **events**:

  - Keyboard, mouse, gamepad (via Gilrs)
  - Window system events (via Winit: resize, close, focus, etc.)

Semantics are identical to `vulfram_receive_queue`, but the content is
strictly _event_ data, not generic messages.

Typical flow:

1. The core collects events during `vulfram_tick()`.
2. The host calls `vulfram_receive_events()`.
3. Binding copies and frees the buffer.
4. Binding decodes events and forwards them into the host-side input system.

---

### 2.5 Upload of Raw Blobs

```c
u32 vulfram_upload_buffer(uint64_t id,
                          uint32_t type,
                          const uint8_t* buffer,
                          size_t length);
```

#### 2.5.1 `vulfram_upload_buffer`

Direction: **Host → Core**

Parameters:

- `id`
  `BufferId` chosen by the host. Used to reference this upload in later commands.

- `type`
  Numeric enum representing the kind of upload, e.g.:

  - shader source / bytecode
  - vertex data
  - index data
  - texture image
  - other binary assets

- `buffer`, `length`
  Pointer and size of the raw data.

Behavior:

- The core **copies** the contents into its internal upload table.
- Later, commands (via `send_queue`) such as `CreateShader`, `CreateTexture`, etc.
  will look up these uploads by `BufferId` and `type`.
- Uploads are treated as **one-shot**: once consumed by a `Create*` command, they
  may be removed from the upload table.

### 2.6 Tick / Frame Advance

```c
u32 vulfram_tick(uint64_t time, uint32_t delta_time);
```

Called **once per frame** by the host.

Parameters:

- `time`
  Host-provided time (currently forwarded to shaders).

- `delta_time`
  Host-provided delta time (currently forwarded to shaders).

Core responsibilities in `tick`:

- Process pending commands enqueued via `send_queue`.
- Update internal component state (transforms, camera matrices, etc.).
- Collect input & window events from Winit / Gilrs.
- Execute the render pipeline on the GPU using WGPU.
- Populate internal queues for:

  - messages (`receive_queue`)
  - events (`receive_events`)
  - profiling data (`get_profiling`)

---

### 2.7 Profiling

```c
u32 vulfram_get_profiling(uint8_t** out_ptr, size_t* out_length);
```

- Returns a MessagePack buffer containing **profiling information**:

  - Timing of internal sections (tick, render passes, uploads, etc.)
  - Counters (draw calls, instance count, resource counts, etc.)

Usage patterns:

- **Continuous debug mode**: call each frame to build an in-app debug overlay.
- **On-demand**: call only when the user/dev requests profiling info.

Contract:

- Same as other `out_ptr` functions:

  - binding copies the data, frees the buffer, then decodes MessagePack.

---

## 3. Recommended Frame Loop Contract

While the core is host-agnostic, we recommend the following call order
per frame on the main thread:

1. **Prepare game logic** (host side)

   - Run ECS / game systems.
   - Decide which components/resources need to be created/updated.
   - Optionally prepare uploads (meshes, textures, etc.).

2. **Uploads** (optional, zero or more calls)

   - For each heavy blob:

     - `vulfram_upload_buffer(buffer_id, type, ptr, length)`

3. **Send commands**

   - Build a MessagePack batch of commands (create/update components & resources,
     maintenance commands).
   - `vulfram_send_queue(buffer, length)`

4. **Advance the core**

   - `vulfram_tick(time, delta_time)`

5. **Receive messages** (consumes response queue)

   - `vulfram_receive_queue(&ptr, &len)`
   - If `len > 0`:

     - copy & free buffer
     - decode MessagePack and process messages

6. **Receive events**

   - `vulfram_receive_events(&ptr, &len)`
   - If `len > 0`:

     - copy & free buffer
     - decode MessagePack and feed the host’s input/window system

7. **Profiling (optional)**

   - `vulfram_get_profiling(&ptr, &len)`
   - If `len > 0`:

     - copy & free buffer
     - decode and feed debug UI / logs

---

## 4. Error Handling Guidelines for Bindings

- On **non-zero** return from any `vulfram_*` function:

  - The binding should surface an error in the host language.

- For debuggability, the core may emit additional structured error
  info into `receive_queue`, which the host can poll and log/visualize.

A typical pattern:

1. Binding call fails (non-zero `VulframResult`).
2. Binding throws/returns an error.
3. Host can additionally call `vulfram_receive_queue` to fetch
   detailed error messages, if desired.
