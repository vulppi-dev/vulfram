# 🦊 Vulfram — Architecture, Lifecycle and Main Loop

This document explains how the Vulfram core is structured at a high level,
how its lifecycle works, and how the host is expected to drive the main loop.

---

## 1. High-Level Architecture

Conceptual data flow:

> **Host** → (commands & uploads) → **Vulfram Core** → **WGPU / GPU**

### 1.1 Host Responsibilities

The host is any runtime that calls the C-ABI functions (or WASM exports), for example:

- Node.js (N-API)
- Lua
- Python
- Any other FFI-capable environment
- Browser runtimes via WASM (WebGPU + DOM canvas)

The host is responsible for:

- Managing the **game logic** and world state.
- Generating **logical IDs** (window, camera, model, light, geometry, material, texture, etc.).
- Building **MessagePack command batches** and sending them to the core.
- Feeding time (`time`, `delta_time`) into `vulfram_tick`.
- Reading **events** and **responses** from the core and reacting to them.

The host does **not**:

- Create windows manually (handled by the core via platform proxies).
- Talk to GPU APIs directly.
- Manage WGPU devices, queues, or pipelines.

### 1.2 Core Responsibilities

The core is the Rust dynamic library that implements Vulfram.
It uses:

- `wgpu` for rendering (WebGPU)
- `winit` for native window + OS events
- `gilrs` for native gamepad input
- `web-sys` for browser window/input plumbing (WASM)
- `image` for texture decoding
- `glam` + `bytemuck` for math and buffer packing
- `serde` + `rmp-serde` for MessagePack

Core responsibilities:

- Keep track of **resources**:
  - Geometries, materials, textures (and shadows).
- Keep track of **instances** (components) per host ID:
  - Cameras, models, lights.
- Manage GPU buffers, textures, pipelines, and render passes.
- Collect and expose input/window events via platform proxies.
- Perform rendering in `vulfram_tick`.

---

## 2. Components, Resources and Instances

### 2.1 Components

Components represent high-level logic and are attached to entities:

- `Camera`
- `Model` (mesh instance)
- `Light`

They are created and updated via commands in `vulfram_send_queue`.
Each component is associated with a host-chosen ID (e.g. `camera_id`, `model_id`, `light_id`).

### 2.2 Resources

Resources are reusable data assets such as:

- Geometries
- Textures
- Materials

They are referenced from components via **logical IDs**:

- `GeometryId`, `MaterialId`, `TextureId`, etc.

Some data (static, per-component values like local colors or viewports) live inside
the component and are **not** standalone resources.

### 2.3 Internal Instances

Internally, the core maintains per-entity instances like cameras, models, and lights.
These instances hold GPU bindings, visibility masks, and render state derived from
the host payloads.

These internal instances are indexed by host IDs and are not visible to the host.
The host always refers to entities by their logical IDs, and the core resolves that to
its internal instance structures.

---

## 3. Asynchronous Resource Linking (Fallback-Driven)

Vulfram allows resources to be created out of order:

- Models can reference geometry or material IDs that do not exist yet.
- Materials can reference texture IDs that do not exist yet.

When a referenced resource is missing, the core uses fallback resources so
rendering continues. When the real resource appears later with the same ID,
the core picks it up automatically on the next frame.

This enables async streaming, independent loading pipelines, and decoupled
creation order.

## 4. LayerMask and Visibility

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

## 4.1 Resource Reuse Semantics

- A single geometry can be referenced by many models.
- A single material can be referenced by many models.
- A single texture can be referenced by many materials.

There is no ownership tracking. The host is responsible for disposing resources
when no longer needed; if a resource is disposed while still referenced,
rendering falls back gracefully.

---

## 4.2 Render Ordering & Batching (Per Camera)

- Opaque/masked objects are sorted by `(material_id, geometry_id)` to reduce
  state changes and batch draw calls.
- Transparent objects are sorted by depth for correct blending.

Draw calls are batched by runs of `(material_id, geometry_id)` after sorting.

## 4.3 Forward Shading (Standard vs PBR)

- The Standard branch favors cheaper shading; the PBR branch favors realism.
- Light evaluation only runs the relevant path per light kind to avoid wasted work.
- Specular in the Standard branch only applies to directional/point/spot lights.

---

## 5. Core Lifecycle

### 5.1 Startup

1. The host loads the Vulfram dynamic library.

2. The host calls:

   ```c
   vulfram_init();
   ```

3. The core initializes:
   - Platform proxy (desktop or browser)
   - WGPU instance (device/queue created on first window)
   - Gilrs (native gamepad) and web gamepad polling (WASM)
   - Internal resource/component tables
   - Profiling and internal queues

### 5.2 Loading / Initial Configuration

In the loading phase, the host typically:

- Uploads heavy data (meshes, textures) via `vulfram_upload_buffer`.
- Sends one or more command batches via `vulfram_send_queue` to:
  - create resources (`CmdGeometryCreate`, `CmdTextureCreateFromBuffer`, `CmdMaterialCreate`, etc.)
  - create components (`CmdCameraCreate`, `CmdModelCreate`, `CmdLightCreate`, …)

The core processes these commands on subsequent calls to `vulfram_tick`.

### 5.3 Main Loop

Once the initial state is ready, the host enters its main loop, where:

- `vulfram_tick` drives the core each frame and consumes queued commands.
- The host sends updates and receives events/responses.

### 5.4 Shutdown

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

## 6. Recommended Main Loop (Host Side)

The exact structure of the host loop is flexible, but a recommended pattern is:

```text
while (running) {
    1. Update host-side logic
    2. Perform uploads (optional)
    3. Send command batch
    4. Call vulfram_tick (processes queued commands)
    5. Receive responses (consumes response queue)
    6. Receive events
    7. (Optional) Receive profiling
}
```

In more detail:

### 6.1 Update Host Logic

- Compute new game state (ECS systems, scripts, AI, etc.).
- Decide which entities/components/resources need:
  - to be created
  - to be updated
  - to be destroyed (future).

### 6.2 Upload Heavy Data (Optional)

For any new or replaced heavy asset:

- Call:

  ```c
  vulfram_upload_buffer(buffer_id, type, ptr, length);
  ```

- Typical uploaded data:
  - Vertex/index buffers
  - Texture images

These uploads will later be consumed by `Create*` commands referenced by `buffer_id`.

### 6.3 Send Command Batch

- Build a batch of commands describing what changed this frame:
  - Component create/update
  - Resource create/update
  - Maintenance (e.g. `CmdUploadBufferDiscardAll`)

- Serialize this to MessagePack.
- Call:

  ```c
  vulfram_send_queue(buffer, length);
  ```

The core will copy the buffer and queue the commands for processing.

### 6.4 Advance the Core (`vulfram_tick`)

- Call:

  ```c
  vulfram_tick(time, delta_time);
  ```

The core will:

- Process all queued commands.
- Update internal component state (camera matrices, transforms, etc.).
- Collect input/window events.
- Execute rendering using WGPU.
- Fill internal queues for responses, events and profiling.

### 6.5 Receive Responses (Optional)

- Call:

  ```c
  uint8_t* ptr = NULL;
  size_t len = 0;
  vulfram_receive_queue(&ptr, &len);
  ```

- If `len > 0`:
  - Copy the bytes to host memory (JS Buffer / Python bytes / Lua string, etc.).
  - Free the core buffer via the mechanism defined in the binding.
  - Deserialize MessagePack and process the responses.

`vulfram_receive_queue` consumes and clears the internal response queue.

### 6.6 Receive Events

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

### 6.7 Profiling (Optional)

For debug or tooling:

- Call:

  ```c
  uint8_t* ptr = NULL;
  size_t len = 0;
  vulfram_get_profiling(&ptr, &len);
  ```

- If `len > 0`:
  - Copy the bytes.
  - Free the core buffer.
  - Deserialize MessagePack into profiling data.
  - Display or log via in-engine tools, overlays, or external tools.

---

## 7. One-shot Uploads and Cleanup

Heavy binary uploads use `vulfram_upload_buffer` and `BufferId`:

1. The host calls `vulfram_upload_buffer(buffer_id, type, bytes, len)`.
2. The core stores the blob in an internal upload table as:
   `BufferId → { type, bytes, used_flag }`.
3. A `Create*` engine command (via `send_queue`) references `buffer_id` and uses
   its data to create a resource (geometry buffers, textures, etc.).
4. Once consumed, the upload entry is marked as used and can be removed.
5. A maintenance command (`CmdUploadBufferDiscardAll`) may be used to clean up
   any remaining, never-used uploads.

This model:

- Avoids shared `BufferId`s.
- Keeps memory usage predictable.
- Fits well with load phases and streaming scenarios.
