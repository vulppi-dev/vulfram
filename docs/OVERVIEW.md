# 🦊 Vulfram — Overview

Vulfram is a **rendering and systems core** written in Rust and exposed as a dynamic library.
It is designed to be driven by external _hosts_ via FFI:

- Node.js (N-API)
- Node.js (N-API)
- Lua (via `mlua`)
- Python (via `PyO3`)
- Any other environment capable of calling C-ABI functions

The central idea is:

> The host controls the engine **only** through:
>
> - a small set of C functions (`vulfram_*`), and
> - binary buffers serialized with **MessagePack**.

The core remains a **black box** that owns the windowing, input, GPU resources, and render pipeline.

---

## 1. Design Goals

- **Host-agnostic**  
  The core does not assume ECS, OOP, or any specific game framework.  
  The host can be an ECS, a custom game loop, scripting, or any mix.

- **Minimal public surface**  
  Only a handful of C-ABI functions are exposed. Everything else is driven by
  data in MessagePack buffers (commands, events, profiling).

- **Binary and fast**  
  All structured communication uses MessagePack (via `serde` + `rmp-serde`).  
  Heavy data (meshes, textures, shaders, etc.) is sent as raw byte blobs.

- **Separation of responsibilities**
  - Host: game logic, world state, IDs, high-level decisions.
  - Core: GPU, window, input, resource management, render pipeline.

---

## 2. High-Level Architecture

Conceptual flow:

> **Host** → (commands & uploads) → **Vulfram Core** → **WGPU / GPU**

### 2.1 Host

The host is the code that calls the `vulfram_*` functions.
Typical hosts:

- Node.js (N-API)
- Lua (via `mlua`)
- Python (via `PyO3`)

The host is responsible for:

- Generating **logical IDs**:
  - `ComponentId`
  - `ShaderId`
  - `GeometryId`
  - `MaterialId`
  - `TextureId`
  - `BufferId` (for uploads)
- Building command batches in MessagePack.
- Calling the ABI functions in the correct order (loop).
- Integrating events (input, window) into its own logic.
- Consuming responses via `vulfram_receive_queue` (clears the response queue).

The host never needs to know about:

- GPU APIs (Vulkan/Metal/DX/etc.)
- WGPU internals
- Winit window/event APIs
- Pipeline / bind group layouts

### 2.2 Vulfram Core

The core is implemented in Rust and uses:

- `winit` for window creation and OS-level events
- `wgpu` for rendering (cross-platform GPU abstraction)
- `gilrs` for gamepad input
- `image` for texture decoding
- `glam` + `bytemuck` for math and safe binary packing
- `serde`, `serde_repr`, `rmp-serde` for MessagePack serialization

The core is responsible for:

- Tracking **resources** (shaders, materials, textures, geometries…)
- Tracking **component instances** (cameras, models, etc.) per `ComponentId`
- Managing GPU buffers and pipelines
- Collecting input and window events
- Executing the render pipeline in `vulfram_tick`

### 2.3 GPU Layer

Below the core, WGPU manages the actual GPU resources:

- Vertex and index buffers
- Uniform and storage buffers
- Textures and samplers
- Render pipelines and render passes

The host never touches this directly. It only refers to logical IDs
and sends commands that the core translates into WGPU operations.

---

## 3. Components and Resources

Vulfram uses two key concepts for scene description:

- **Components** (complex, high-level state attached to entities)
- **Resources** (reusable data assets)

### 3.1 Components

Components represent “what exists in the scene” and how it behaves.
They are always associated with an `ComponentId` chosen by the host.

Examples of components:

- `CameraComponent` (with `order` and optional `view_position`)
- `ModelComponent` (transform data; references `geometry_id`, future `material_id`)
- `LightComponent` (future)
- `EnvironmentComponent` (future)

Components may contain:

- **Static data**: values that live _inside_ that component only
  (colors, matrices, viewport info, etc.).
- **References to sharable resources**:
  via logical IDs (e.g. `MaterialId`, `GeometryId`, `TextureId`).

The host creates and updates components through commands in the
`send_queue` MessagePack buffer.

### 3.2 Resources

Resources are the underlying data used by components.

Examples:

- Shaders (future)
- Geometries (current)
- Textures (future)
- Materials (future)
- Samplers (future)
- (Future: fonts, sounds, etc.)

They are split into two categories:

#### 3.2.1 Sharable Resources

- Can be shared among multiple components/entities.
- Identified by **logical IDs** visible to the host:
  - `ShaderId`, `GeometryId`, `MaterialId`, `TextureId`, etc.
- Internally, the core maps these IDs to GPU handles using internal “holders”.

#### 3.2.2 Static Resources

- Exist only **inside** a specific component.
- Have no standalone logical ID.
- Carried in the component payload itself.
- Typical examples:
  - Per-instance colors
  - Per-instance matrices
  - Camera viewport config

---

## 4. IDs and Internal Handles

### 4.1 Logical IDs (visible to the host)

The host generates and owns:

- `CameraId` — identifies a camera
- `ModelId` — identifies a model instance
- `GeometryId` — mesh/geometry asset
- `MaterialId` — material asset (future)
- `TextureId` — texture asset (future)
- `BufferId` — upload blob identifier

---

## 4.3 Asynchronous Resource Linking (Fallback-Driven)

Vulfram allows resources to be created out of order:

- Models can reference geometry or material IDs that do not exist yet.
- Materials can reference texture IDs that do not exist yet.

When a referenced resource is missing, the core uses fallback resources
so rendering continues. When the real resource appears later with the same ID,
the core picks it up automatically on the next frame.

This enables async streaming, independent loading pipelines, and
decoupled creation order.

These are simple integers from the core’s perspective. The only rule is:

- The host must not reuse an ID for different purposes unless a
  well-defined destroy/replace protocol is in place.

### 4.2 Internal Handles (core-only)

Inside the core, logical IDs are resolved into internal handles:

- `ShaderModuleHandle`
- `RenderPipelineHandle`
- `BufferHandle` (GPU buffer)
- `TextureHandle`
- `SamplerHandle`

- `CameraInstanceHandle`
- `MeshInstanceHandle`
- (future) `LightInstanceHandle`, `EnvironmentInstanceHandle`, etc.

These handles are never exposed to the host. They are internal indices,
pointers, or IDs used to drive WGPU objects.

---

## 5. One-shot Uploads and Resource Creation

Heavy data is sent via `vulfram_upload_buffer` using `BufferId`s.

Typical flow:

1. Host calls `vulfram_upload_buffer(buffer_id, type, bytes, len)` one or more times.
2. Host sends commands (in `send_queue`) like `CreateShader`, `CreateGeometry`,
   `CreateTexture`, `CreateMaterial` that **reference those `BufferId`s**.
3. The core:
   - looks up each `BufferId` in its internal upload table
   - creates the GPU resources (shader modules, buffers, textures)
   - binds those resources to logical IDs (`ShaderId`, `GeometryId`, …)
   - marks uploads as consumed/removed
4. A maintenance command (e.g. `DiscardUnusedUploads`) may be used to
   free any upload blobs that were never consumed.

This enforces:

- **One-shot semantics** for `BufferId`s (they are not shared).
- Clear memory lifetime for upload blobs.

---

## 6. Layers and Visibility (LayerMask)

To control what is visible where, Vulfram uses bitmask layers (`u32`):

- Camera components have `layerMaskCamera: u32`
- Model components have `layerMaskComponent: u32`
- (future) Lights may have `layerMaskLight: u32`

Visibility rule for models:

```text
A model is visible in a camera if:

    (layerMaskCamera & layerMaskComponent) > 0
```

This supports:

- World-only cameras
- UI-only cameras
- Team- or group-based filtering
- Special passes (e.g. picking, debug)

---

## 6.1 Resource Reuse Semantics

- A single geometry can be referenced by many models.
- A single material can be referenced by many models.
- A single texture can be referenced by many materials.

There is no ownership tracking. If a resource is disposed while still referenced,
rendering falls back gracefully.

---

## 6.2 Render Ordering & Batching (Per Camera)

- Opaque/masked objects are sorted by `(material_id, geometry_id)` to reduce
  state changes and batch draw calls.
- Transparent objects are sorted by depth for correct blending.

Draw calls are batched by runs of `(material_id, geometry_id)` after sorting.

---

## 7. What the Host Sees vs. What the Host Does Not

### 7.1 Host sees

- A small set of **C functions** (`vulfram_*`)
- MessagePack format for:

  - commands
  - responses/messages
  - events
  - profiling

- Logical IDs and their own structures en/decoded on the host side.

### 7.2 Host does **not** see

- Internal Rust types
- WGPU device/queue/pipelines/bind groups
- Winit windows/surfaces
- Gilrs details
- Internal handles and instance structures

---

## 8. Recommended Reading Order

For engine users (binding authors or advanced users):

1. `docs/OVERVIEW.md`
2. `docs/ABI.md`
3. `docs/ARCH.md`

For engine contributors (Rust core developers):

1. `docs/OVERVIEW.md`
2. `docs/ARCH.md`
3. `docs/API.md`
4. `docs/GLOSSARY.md`
