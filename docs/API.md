# 🦊 Vulfram — Internal API (Crates, Structs, Internal Flow)

> This is an **internal engineering document**.
> It describes how the Vulfram core is structured on the Rust side: crates,
> data structures, and internal flows. It is **not** intended for engine users.

---

## 1. Crates and Runtime Stack

The Vulfram core is built as a Rust library with the following key crates:

- **Windowing & OS events**

  - `winit`
    - Window creation
    - Event loop integration
    - Input events (keyboard, mouse)

- **Rendering**

  - `wgpu`
    - Cross-platform GPU abstraction
    - Device and queue management
    - Render pipelines, buffers, textures

- **Gamepad input**

  - `gilrs`
    - Gamepad discovery and events

- **Images**

  - `image`
    - Texture file decoding (PNG, JPEG, etc.)
    - Conversion to raw pixel data for WGPU

- **Math / binary packing**

  - `glam`
    - Vector, matrix, quaternion types (`Vec2`, `Vec3`, `Mat4`, etc.)
  - `bytemuck`
    - Safe casting between typed structs and `&[u8]` / `&mut [u8]`
    - Marks types as `Pod` / `Zeroable` for GPU upload

- **Serialization**
  - `serde`
  - `serde_repr`
  - `rmp-serde`
    - MessagePack serialization/deserialization for commands, events, profiling

---

## 2. Engine State Overview

At a high level, the core is organized around a central `EngineState`:

```rust
pub struct EngineState {
    // Subsystems
    window_system: WindowSystem,        // winit integration
    render_system: RenderSystem,        // wgpu integration
    input_system: InputSystem,          // winit + gilrs

    // Resource and component tables
    resources: Resources,
    components: Components,

    // Queues
    cmd_queue: InternalCmdQueue,        // commands from host (decoded MessagePack)
    msg_queue: InternalMsgQueue,        // generic messages to host
    event_queue: InternalEventQueue,    // events to host (input/window)

    // Uploads
    buffers: HashMap<u64, UploadBuffer>, // BufferId -> UploadBuffer

    // Profiling
    profiling_data: ProfilingData,
}
```

---

## 3. Resources

`Resources` holds all sharable resources indexed by logical IDs:

```rust
pub struct Resources {
    shaders:    HashMap<ShaderId, ShaderResource>,
    geometries: HashMap<GeometryId, GeometryResource>,
    materials:  HashMap<MaterialId, MaterialResource>,
    textures:   HashMap<TextureId, TextureResource>,
    // future: fonts, sounds, etc.
}
```

### 3.1 ShaderResource

Contains:

- `shader_id: ShaderId`
- `module: wgpu::ShaderModule`
- Optional reflection or metadata if needed for pipeline creation.

### 3.2 GeometryResource

Represents a mesh/geometry asset:

- One or more `wgpu::Buffer` for vertex data (or a shared buffer + ranges).
- One `wgpu::Buffer` for index data (optional, depending on topology).
- Layout description:

  - per-vertex attribute formats
  - strides, offsets
  - index format and range

### 3.3 TextureResource

Wraps:

- `wgpu::Texture`
- `wgpu::TextureView` for sampling / attachments
- Possibly a `wgpu::Sampler` or separate sampler resource.

### 3.4 MaterialResource

Represents everything needed to draw with a given material:

- Pipeline:

  - `PipelineSpec` (logical description: shader, blend, depth, primitive, etc.)
  - `wgpu::RenderPipeline` (compiled/cached pipeline)

- Uniform data:

  - Offset into `MaterialUniformBuffer` (per-material constants)

- Storage data (optional):

  - Offset into `MaterialStorageBuffer` for large per-material data

- Textures & samplers:

  - List of `TextureHandle`s and sampler handles to bind.

---

## 4. Components and Instances

The `Components` struct maps `ComponentId` to internal instances:

```rust
pub struct Components {
    cameras: HashMap<ComponentId, CameraInstance>,
    models:  HashMap<ComponentId, MeshInstance>,
    // future: lights, environments...
}
```

### 4.1 CameraInstance

A camera instance stores:

- `camera_uniform_offset: u32`

  - Index into `CameraUniformBuffer` for its `CameraPW` (projection \* view matrix).

- `viewport: Viewport`

  - Mode (relative/absolute)
  - x, y, width, height

- `render_target: TextureHandle`

  - Dedicated render target texture for this camera.

- `layer_mask_camera: u32`

  - Layer mask used for visibility filtering.

### 4.2 MeshInstance

A mesh/model instance stores:

- `geometry: GeometryHandle`

  - Link to a `GeometryResource`.

- `material: MaterialHandle`

  - Link to a `MaterialResource`.

- `model_uniform_offset: u32`

  - Index into `ModelUniformBuffer` for its transform matrix (model).

- `layer_mask_component: u32`

  - Layer mask used by cameras for visibility filtering.

---

## 5. Internal Command Flow

Commands are sent from the host via MessagePack to `vulfram_send_queue`,
then decoded into internal Rust enums.

### 5.1 ABI Layer (C → Rust)

- `vulfram_send_queue(buffer, length)`

  1. Copies the buffer into a `Vec<u8>`.
  2. Deserializes with `rmp-serde` into a `Vec<EngineCommand>`.
  3. Pushes the commands into `EngineState::cmd_queue`.

### 5.2 Command Representation

Conceptual example:

```rust
enum EngineCommand {
    // Resource creation
    CreateShader {
        shader_id: ShaderId,
        buffer_id: BufferId,
    },
    CreateGeometry {
        geometry_id: GeometryId,
        buffers: GeometryBuffers,  // buffer IDs + layout info
    },
    CreateTexture {
        texture_id: TextureId,
        buffer_id: BufferId,
        params: TextureParams,
    },
    CreateMaterial {
        material_id: MaterialId,
        shader_id: ShaderId,
        textures: Vec<TextureId>,
        params: MaterialParams,
    },

    // Component creation
    CreateCameraComponent {
        component_id: ComponentId,
        desc: CameraDesc,
        viewport: ViewportDesc,
        layer_mask: u32,
    },
    CreateModelComponent {
        component_id: ComponentId,
        geometry_id: GeometryId,
        material_id: MaterialId,
        layer_mask: u32,
    },

    // Component updates
    UpdateCameraComponent {
        component_id: ComponentId,
        camera_pw: glam::Mat4,
        viewport: Option<ViewportDesc>,
    },
    UpdateModelTransform {
        component_id: ComponentId,
        model: glam::Mat4,
    },

    // Maintenance
    DiscardUnusedUploads,

    // Future: destroy commands, lights, environment, etc.
}
```

### 5.3 Command Execution

During `EngineState::tick` (called from `vulfram_tick`):

1. Drain `cmd_queue`.
2. For each `EngineCommand`, call into appropriate systems:

   - Resource creation/update
   - Component creation/update
   - Maintenance (e.g. cleaning uploads)

3. Update any derived data required for rendering (culling, visibility, etc.).

---

## 6. Upload Handling (`UploadBuffer`)

Upload buffers manage blobs uploaded via `vulfram_upload_buffer`:

```rust
pub struct UploadBuffer {
    pub upload_type: UploadType,
    pub data: Vec<u8>,
}
```

Stored in `EngineState`:

```rust
buffers: HashMap<u64, UploadBuffer>  // BufferId -> UploadBuffer
```

- `vulfram_upload_buffer`:

  - Inserts an `UploadBuffer` for a given `BufferId` (u64).
  - Returns error if buffer ID already exists (one-shot semantics).

- `Create*` commands:

  - Look up the `UploadBuffer` by `BufferId`.
  - Use/consume its data to create WGPU resources.
  - Remove entry after consumption.

- `DiscardUnusedUploads` command:

  - Iterates and removes any unconsumed upload buffers.

---

## 7. Rendering System Overview

The `RenderSystem` is responsible for managing WGPU objects and executing
the draw passes.

### 7.1 Buffers

Typical GPU buffers:

- `CameraUniformBuffer`

  - Array of camera uniforms (projection \* view matrix, etc.).

- `ModelUniformBuffer`

  - Array of model transforms.

- `MaterialUniformBuffer`

  - Array of material constants.

- `MaterialStorageBuffer`

  - Optional large data per material.

- Vertex / index buffers for geometries.

### 7.2 Render Pass Flow (per Frame)

Conceptual flow:

1. Update uniform/storage buffers (if dirty):

   - Write updated camera/model/material data into mapped ranges.

2. For each camera:

   - Set its render target (texture view) as the color attachment.
   - Configure viewport and scissor according to `CameraInstance`.
   - Clear or load the attachment as needed.
   - Iterate over `MeshInstance`s:

     - Filter by `layerMask`:

       ```rust
       if camera.layer_mask & mesh.layer_mask != 0 {
           // visible
       }
       ```

     - Select material and pipeline:

       - Fetch pipeline from `MaterialResource` or create/cache if needed.

     - Bind vertex/index buffers and resource bind groups.

     - Issue draw calls via `RenderPass::draw` / `draw_indexed`.

3. (Future) Composite camera render targets into a final swapchain image.

---

## 8. Event System

The `InputSystem` is responsible for aggregating events:

- Keyboard & mouse from `winit`
- Gamepad from `gilrs`
- Window events (resize, close, focus, etc.) from `winit`

These are translated into internal `EngineEvent` enums and pushed into
`event_queue`.

On `vulfram_receive_events`, the core:

1. Serializes `event_queue` into MessagePack (using `rmp-serde`).
2. Allocates a buffer, copies bytes, and exposes pointer & length.
3. Clears `event_queue` for the next frame.

---

## 9. Profiling Data

`ProfilingData` tracks performance metrics, such as:

- Section timings:

  - `tick_total`
  - `render_total`
  - `command_processing`
  - `event_collection`
  - etc.

- Counters:

  - number of draw calls
  - number of visible mesh instances
  - number of active resources

On `vulfram_profiling`, the core:

1. Takes a snapshot of `ProfilingData`.
2. Serializes it into MessagePack.
3. Allocates and exposes the buffer via pointer & length.
