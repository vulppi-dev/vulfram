# 🦊 Vulfram — Architecture, Lifecycle and Main Loop

This document explains how the Vulfram core is structured at a high level,
how its lifecycle works, and how the host is expected to drive the main loop.

---

## 1. High-Level Architecture

Conceptual data flow:

> **Host** → (commands & uploads) → **Vulfram Core** → **WGPU / GPU**

### 1.1 Host Responsibilities

The host is any runtime that calls the C-ABI functions, for example:

- Node.js / Bun
- Lua
- Python
- Any other FFI-capable environment

The host is responsible for:

- Managing the **game logic** and world state.
- Generating **logical IDs** (entities, materials, textures, etc.).
- Building **MessagePack command batches** and sending them to the core.
- Feeding time (`time`, `delta_time`) into `vulfram_tick`.
- Reading **events** and **messages** from the core and reacting to them.

The host does **not**:

- Create windows manually (handled via Winit inside the core).
- Talk to GPU APIs directly.
- Manage WGPU devices, queues, or pipelines.

### 1.2 Core Responsibilities

The core is the Rust dynamic library that implements Vulfram.
It uses:

- `winit` for window + OS events
- `wgpu` for rendering
- `gilrs` for gamepad input
- `image` for texture decoding
- `glam` + `bytemuck` for math and buffer packing
- `serde` + `rmp-serde` for MessagePack

Core responsibilities:

- Keep track of **resources**:
  - Shaders, geometries, materials, textures, samplers…
- Keep track of **instances** (components) per `EntityId`:
  - Cameras, models, etc.
- Manage GPU buffers, textures, pipelines, and render passes.
- Collect and expose input/window events.
- Perform rendering in `vulfram_tick`.

---

## 2. Components, Resources and Instances

### 2.1 Components

Components represent high-level logic and are attached to entities:

- `CameraComponent`
- `ModelComponent` (mesh instance)
- (future) `LightComponent`
- (future) `EnvironmentComponent`

They are created and updated via commands in `send_queue`.
Each component is associated with an `EntityId` chosen by the host.

### 2.2 Resources

Resources are reusable data assets such as:

- Shaders
- Geometries
- Textures
- Materials
- Samplers

They are referenced from components via **logical IDs**:

- `ShaderId`, `GeometryId`, `MaterialId`, `TextureId`, etc.

Some data (static, per-component values like local colors or viewports) live inside
the component and are **not** standalone resources.

### 2.3 Internal Instances

Internally, the core maintains per-entity instances like:

- `CameraInstance`

  - A slot in `CameraUniformBuffer`
  - Viewport data
  - A dedicated render target texture
  - `layerMaskCamera`

- `MeshInstance`
  - References to `GeometryResource` and `MaterialResource`
  - A slot in `ModelUniformBuffer`
  - `layerMaskComponent`

These internal instances are indexed by `EntityId` and are not visible to the host.
The host always refers to entities by `EntityId`, and the core resolves that to
its internal instance structures.

---

## 3. LayerMask and Visibility

The core uses a `u32` bitmask to filter visibility:

- Each camera has a `layerMaskCamera`.
- Each model/mesh has a `layerMaskComponent`.
- (Future) Each light may have a `layerMaskLight`.

Visibility rule for a given camera/model pair:

```text
Visible if:

    (layerMaskCamera & layerMaskComponent) > 0
```

This enables:

- World-only or UI-only cameras.
- Team or category-based rendering.
- Dedicated special passes (e.g. picking, debug-only geometry).

---

## 4. Core Lifecycle

### 4.1 Startup

1. The host loads the Vulfram dynamic library.

2. The host calls:

   ```c
   vulfram_init();
   ```

3. The core initializes:

   - Winit (window, event loop integration)
   - WGPU (instance, device, queue)
   - Gilrs (gamepad)
   - Internal resource/component tables
   - Profiling and internal queues

### 4.2 Loading / Initial Configuration

In the loading phase, the host typically:

- Uploads heavy data (meshes, textures, shaders) via `vulfram_upload_buffer`.
- Sends one or more command batches via `vulfram_send_queue` to:

  - create resources (`CreateShader`, `CreateTexture`, `CreateMaterial`, etc.)
  - create components (`CreateCameraComponent`, `CreateModelComponent`, …)

The core processes these commands on subsequent calls to `vulfram_tick`.

### 4.3 Main Loop

Once the initial state is ready, the host enters its main loop, where:

- `vulfram_tick` drives the core each frame.
- The host sends updates and receives events/messages.

### 4.4 Shutdown

When the application is closing:

1. The host stops calling `vulfram_tick`.

2. The host calls:

   ```c
   vulfram_dispose();
   ```

3. The core releases:

   - GPU resources
   - Window and OS handles
   - Internal allocations

---

## 5. Recommended Main Loop (Host Side)

The exact structure of the host loop is flexible, but a recommended pattern is:

```text
while (running) {
    1. Update host-side logic
    2. Perform uploads (optional)
    3. Send command batch
    4. Call vulfram_tick
    5. Receive messages
    6. Receive events
    7. (Optional) Receive profiling
}
```

In more detail:

### 5.1 Update Host Logic

- Compute new game state (ECS systems, scripts, AI, etc.).
- Decide which entities/components/resources need:

  - to be created
  - to be updated
  - to be destroyed (future).

### 5.2 Upload Heavy Data (Optional)

For any new or replaced heavy asset:

- Call:

  ```c
  vulfram_upload_buffer(buffer_id, type, ptr, length);
  ```

- Typical uploaded data:

  - Shader source/bytecode
  - Vertex/index buffers
  - Texture images

These uploads will later be consumed by `Create*` commands referenced by `buffer_id`.

### 5.3 Send Command Batch

- Build a batch of commands describing what changed this frame:

  - Component create/update
  - Resource create/update
  - Maintenance (e.g. `DiscardUnusedUploads`)

- Serialize this to MessagePack.
- Call:

  ```c
  vulfram_send_queue(buffer, length);
  ```

The core will copy the buffer and queue the commands for processing.

### 5.4 Advance the Core (`vulfram_tick`)

- Call:

  ```c
  vulfram_tick(time, delta_time);
  ```

The core will:

- Process all queued commands.
- Update internal component state (camera matrices, transforms, etc.).
- Collect input/window events.
- Execute rendering using WGPU.
- Fill internal queues for messages, events and profiling.

### 5.5 Receive Messages (Optional)

- Call:

  ```c
  uint8_t* ptr = NULL;
  size_t len = 0;
  vulfram_receive_queue(&ptr, &len);
  ```

- If `len > 0`:

  - Copy the bytes to host memory (JS Buffer / Python bytes / Lua string, etc.).
  - Free the core buffer via the mechanism defined in the binding.
  - Deserialize MessagePack and process the messages.

### 5.6 Receive Events

- Call:

  ```c
  uint8_t* ptr = NULL;
  size_t len = 0;
  vulfram_receive_events(&ptr, &len);
  ```

- If `len > 0`:

  - Copy the bytes.
  - Free the core buffer.
  - Deserialize MessagePack into an event list.
  - Integrate these into the host’s own input/window systems.

### 5.7 Profiling (Optional)

For debug or tooling:

- Call:

  ```c
  uint8_t* ptr = NULL;
  size_t len = 0;
  vulfram_profiling(&ptr, &len);
  ```

- If `len > 0`:

  - Copy the bytes.
  - Free the core buffer.
  - Deserialize MessagePack into profiling data.
  - Display or log via in-engine tools, overlays, or external tools.

---

## 6. One-shot Uploads and Cleanup

Heavy binary uploads use `vulfram_upload_buffer` and `BufferId`:

1. The host calls `vulfram_upload_buffer(buffer_id, type, bytes, len)`.
2. The core stores the blob in an internal upload table as:
   `BufferId → { type, bytes, used_flag }`.
3. A `Create*` engine command (via `send_queue`) references `buffer_id` and uses
   its data to create a resource (`ShaderResource`, `TextureResource`, etc.).
4. Once consumed, the upload entry is marked as used and can be removed.
5. A logical command such as `DiscardUnusedUploads` may be used to clean up
   any remaining, never-used uploads.

This model:

- Avoids shared `BufferId`s.
- Keeps memory usage predictable.
- Fits well with load phases and streaming scenarios.
