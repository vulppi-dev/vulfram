# 🦊 Vulfram — Glossary (Terminology and Naming Conventions)

> Internal reference for common terminology and naming patterns used in the
> Vulfram project. This is useful for both core contributors and binding authors.

---

## 1. Core Concepts

### Host

Any external program or runtime that calls the `vulfram_*` functions.

Examples:

- Node.js (N-API) runtime
- Lua VM
- Python interpreter
- Custom native applications

Responsibilities of the host:

- Drive the main loop.
- Provide `time` and `delta_time` to `vulfram_tick`.
- Manage game logic and world state (ECS, OOP, etc.).
- Generate and maintain logical IDs (entities, resources, buffers).
- Build MessagePack command batches and send them to the core.
- Consume events, messages, and profiling data from the core.

### Core

The Rust dynamic library implementing Vulfram.

Responsibilities of the core:

- Abstract away window, input, and rendering systems.
- Manage GPU resources and pipelines using WGPU.
- Manage component instances (cameras, models, etc.).
- Translate commands from the host into internal state changes.
- Render frames in `vulfram_tick`.

### ABI (Application Binary Interface)

The C-ABI interface exposed by the core, consisting of:

- Function signatures such as `u32 vulfram_init(void)`.
- Primitive types (`u32`, pointers, `size_t`, `double`).
- No Rust-specific constructs (`String`, traits, generics) cross the ABI boundary.

Bindings (N-API modules, PyO3 extensions, etc.) use this ABI to build
higher-level, language-friendly APIs.

---

## 2. Components vs Resources

### Component

A **component** is a high-level structure describing some behavior or
participation in the scene, usually attached to a `ComponentId`.

Examples:

- `CameraComponent`: Projection, view matrices, viewport, layer mask.
- `ModelComponent`: References to geometry/material, world transform, shadow flags.
- `LightComponent`: Color, intensity, range, type (point, directional, spot).

Characteristics:

- Components are created/updated via commands (MessagePack).
- They can embed **static data**.
- They reference **sharable resources** by logical ID.

### Resource

A **resource** is a reusable asset or configuration used by components.

Examples:

- `GeometryResource`: Vertex and index data.
- `MaterialResource`: Surface properties (Standard or PBR).
- `TextureResource`: Image data (decodded or raw).
- `LightResource`: (Implicitly managed via LightComponent IDs).
- `CameraResource`: (Implicitly managed via CameraComponent IDs).

Resources are identified by logical IDs such as:

- `GeometryId`, `MaterialId`, `TextureId`.

### Label

A **label** is an optional semantic name (string) assigned to a resource or component.

Characteristics:

- Used primarily for debugging and scene discovery.
- Displayed in tools (profiling, debug view) to help identify resources.
- Not used for indexing or internal logic; logical IDs are the primary keys.

#### Sharable Resources

- May be shared between multiple components/entities.
- Are referenced via logical IDs.
- Have internal GPU handles (e.g., `wgpu::Buffer`, `wgpu::Texture`).

#### Static Resources

- Live **inside** a specific component only.
- Not assigned a separate logical ID.
- Serialized as part of the component’s payload.

Example:

- Camera viewport stored directly in `CameraComponent`.
- Instance-specific color in `ModelComponent`.

---

## 3. IDs and Handles

### Logical IDs (Host-visible)

Integers defined and managed by the host. Common logical IDs:

- `ComponentId`
- `ShaderId`
- `GeometryId`
- `MaterialId`
- `TextureId`
- `BufferId` (for uploads)

Convention:

- Logical IDs are **opaque** to the core. They are just keys.
- The host must ensure they are unique and consistently reused or destroyed
  according to the application design.

### Handles (Core-only)

Internal references used by the core, such as:

- `ShaderModuleHandle`
- `RenderPipelineHandle`
- `BufferHandle` (GPU buffer)
- `TextureHandle`
- `SamplerHandle`

And for instances:

- `CameraInstanceHandle`
- `MeshInstanceHandle`
- (future) `LightInstanceHandle`, `EnvironmentInstanceHandle`

These handles are typically indices or pointers managed by the core and are
never exposed through the ABI.

---

## 4. Uploads and Buffers

### Upload

A raw data blob sent from the host to the core via `vulfram_upload_buffer`.

- Identified by `(BufferId, type)`.
- Stored in an internal upload table as an `UploadEntry`.
- Consumable by `Create*` commands referencing `BufferId`.

Uploads are treated as **one-shot**:

- Once used to create resources, they may be removed.
- Unused uploads can be discarded by a maintenance command like
  `CmdUploadBufferDiscardAll`.

### Fallback Resource

Safe default resource used when a referenced ID does not exist yet
(for example, fallback material or fallback texture view). This allows
rendering to continue while resources are created asynchronously.

### Asynchronous Resource Linking

The ability to create models, materials, geometries, and textures in any
order. Missing references use fallbacks until the real resource appears.

### Resource Reuse

Resources are shareable by design:

- One geometry can be referenced by many models.
- One material can be referenced by many models.
- One texture can be referenced by many materials.

There is no ownership tracking; disposing a resource while still referenced
falls back gracefully.

### Buffer (GPU)

A GPU memory object created via WGPU, typically one of:

- Vertex buffer
- Index buffer
- Uniform buffer
- Storage buffer

These are held via `BufferHandle` internally.

---

## 5. Queues

### Command Queue (`send_queue`)

Logical queue of commands coming from the host:

- Create/update/destroy resources.
- Create/update/destroy components.
- Maintenance actions.

Serialized as MessagePack and passed to `vulfram_send_queue`.

Commands are queued and consumed during `vulfram_tick`.

### Message Queue (`receive_queue`)

Logical queue of messages from the core:

- Acknowledgments.
- Error details.
- Debug/log messages (structured).

The host reads this via `vulfram_receive_queue` and decodes MessagePack.

Calling `vulfram_receive_queue` consumes and clears the internal response queue.

### Event Queue (`receive_events`)

Logical queue of events:

- Keyboard/mouse input.
- Gamepad events.
- Window events (resize, focus, close, etc.).

The host reads this via `vulfram_receive_events` and integrates it into its
own input and windowing logic.

---

## 6. LayerMask

`LayerMask` is a `u32` bitmask used to filter visibility and influence.

Common roles:

- `layerMaskCamera`
  - Specifies which layers a camera can see.
- `layerMaskComponent`
  - Specifies which layers a model/mesh belongs to.
- (future) `layerMaskLight`
  - Specifies which layers a light affects.

Common rule:

```text
Visible / influenced if (A.layerMask & B.layerMask) > 0
```

---

## 7. Functions and Files

### `vulfram_*` Functions

All public ABI functions are prefixed with `vulfram_`:

- `vulfram_init`
- `vulfram_dispose`
- `vulfram_send_queue`
- `vulfram_receive_queue`
- `vulfram_receive_events`
- `vulfram_upload_buffer`
- `vulfram_tick`
- `vulfram_profiling`

### Documentation Files

- `vulfram_overview.md`

  - High-level summary and concepts.

- `vulfram_abi_spec.md`

  - Functions, ABI details and usage contract.

- `vulfram_architecture_lifecycle.md`

  - Architecture, lifecycle, and main loop contract.

- `vulfram_internal_api.md`

  - Internal Rust API: crates, structs, internal flows.

- `vulfram_glossary.md`

  - This document: terminology and naming patterns.

---

## 8. Profiling

### ProfilingData

Internal structure that collects:

- Timing for core sections:

  - total tick
  - render passes
  - command processing
  - event collection

- Counters:

  - number of draw calls
  - number of visible mesh instances
  - number of active resources

Exposed to the host via:

- `vulfram_profiling` → MessagePack → host tooling/UI.
