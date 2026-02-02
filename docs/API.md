# 🦊 Vulfram — Internal API (Crates, Structs, Internal Flow)

> This is an **internal engineering document**.
> It describes how the Vulfram core is structured on the Rust side: crates,
> data structures, and internal flows. It is **not** intended for engine users.

---

## 1. Crates and Runtime Stack

The Vulfram core is built as a Rust library with the following key crates:

- **Platform proxies**

  - `winit` (desktop)
    - Window creation
    - Event loop integration
    - Input events (keyboard, mouse, touch, gestures)
  - `web-sys` (browser/WASM)
    - DOM canvas, input events, gamepad polling

- **Rendering**

  - `wgpu`
    - Cross-platform GPU abstraction
    - Device and queue management
    - Render pipelines, buffers, textures

- **Gamepad input**

  - `gilrs` (desktop)
  - Web Gamepad API (browser/WASM)

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
    pub window: WindowManager,

    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub wgpu: wgpu::Instance,
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    pub buffers: BufferStorage,

    pub cmd_queue: EngineBatchCmds,
    pub event_queue: EngineBatchEvents,
    pub response_queue: EngineBatchResponses,

    pub(crate) time: u64,
    pub(crate) delta_time: u32,
    pub(crate) frame_index: u64,

    #[cfg(not(feature = "wasm"))]
    pub input: InputState,
    pub(crate) gamepad: GamepadState,

    pub(crate) profiling: TickProfiling,
}
```

`EngineSingleton` owns the `EngineState` plus a platform proxy
(`DesktopProxy` or `BrowserProxy`) that handles window/input integration.

---

## 3. Resources (Current)

The core manages several first-class resources:

- **Geometry**: Managed by the vertex allocator (pooled or dedicated buffers).
- **Textures**: Loaded from buffers or created as solid colors.
- **Materials**: Define the appearance of meshes.
- **Shadows**: Global shadow mapping configuration per window.

---

## 4. Components and Instances (Current)

The render state keeps a **scene** with camera/model records:

- `CameraRecord`

  - `label: Option<String>` (semantic name)
  - `data: CameraComponent` (projection/view matrices)
  - `layer_mask`, `order`
  - `view_position` (optional, relative/absolute)
  - `render_target` (per-camera texture)

- `ModelRecord`
  - `label: Option<String>` (semantic name)
  - `data: ModelComponent` (transform + derived TRS)
  - `geometry_id` (required)
  - `cast_outline` and `outline_color` (outline mask + color for post)

---

## 5. Environment & Post-Processing (Current)

The environment config now includes a post-processing block used by the `post` pass.

`EnvironmentConfig` (core/resources/environment/spec.rs):

- `msaa`
- `skybox`
- `post`

`PostProcessConfig` highlights:

- `outline_enabled`: enables outline composition in post
- `outline_strength`: mix amount for outline color
- `outline_threshold`: edge threshold (clamped to `[0, 1)`)
- `outline_width`: pixel width used by edge kernel
- `outline_quality`: 0 = 3×3 kernel, 1 = 5×5 kernel

The outline mask is rendered in a dedicated `outline` pass into `outline_color`
(now `rgba8`), and sampled by the `post` pass for final composition.
  - `material_id` (optional)
  - `layer_mask`

The visibility rule uses `layer_mask`:

```text
(model.layer_mask & camera.layer_mask) != 0
```

---

## 5. Internal Command Flow

Commands are sent from the host via MessagePack to `vulfram_send_queue`,
then decoded into internal Rust enums.

### 5.1 ABI Layer (C → Rust)

- `vulfram_send_queue(buffer, length)`

  1. Copies the buffer into a `Vec<u8>`.
  2. Deserializes with `rmp-serde` into `EngineBatchCmds` (`Vec<EngineCmdEnvelope>`).
  3. Pushes the commands into `EngineState::cmd_queue`.

### 5.2 Command Representation

Current command enum (`EngineCmd`) includes:

- **Window**: Create, Close, SetTitle, SetPosition, GetPosition, SetSize, GetSize, GetOuterSize, GetSurfaceSize, SetState, GetState, SetIcon, SetDecorations, HasDecorations, SetResizable, IsResizable, RequestAttention, Focus, SetCursorVisible, SetCursorGrab, SetCursorIcon.
- **Camera**: Create, Update, Dispose, List.
- **Model**: Create, Update, Dispose, List.
- **Light**: Create, Update, Dispose, List.
- **Material**: Create, Update, Dispose, List.
- **Texture**: CreateFromBuffer, CreateSolidColor, Dispose, List.
- **Geometry**: Create, Update, Dispose, List.
- **Primitive Geometry**: Create (Cube, Plane, Sphere, Cylinder, Torus, Pyramid).
- **Shadow**: Configure.
- **Gizmo**: DrawLine, DrawAabb.

### 5.3 Command Execution

During `vulfram_tick`:

1. Drain `cmd_queue`.
2. For each `EngineCommand`, call into appropriate systems:

   - Resource creation/update
   - Component creation/update
   - Maintenance (e.g. cleaning uploads)

3. Update any derived data required for rendering (culling, visibility, etc.).

---

## 5.4 Asynchronous Resource Linking (Fallback-Driven)

The render path tolerates missing references:

- Models may reference geometry or material IDs that do not exist yet.
- Materials may reference texture IDs that do not exist yet.

Missing resources fall back to safe defaults. When the real resource is created
later with the same ID, the core picks it up automatically on the next frame.

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

- `CmdUploadBufferDiscardAll` command:

  - Iterates and removes any unconsumed upload buffers.

---

## 7. Rendering System Overview

The `RenderState` is responsible for managing WGPU objects and executing
the draw passes.

### 7.1 Buffers

Current GPU buffers:

- `FrameUniformBuffer`

  - Time, delta time, frame index.

- `CameraUniformBuffer`

  - Camera matrices and parameters.

- `ModelUniformBuffer`

  - Model transforms and derived TRS.

- Vertex / index buffers for geometries (managed by `VertexAllocatorSystem`).

### 7.2 Render Pass Flow (per Frame)

Conceptual flow:

1. Update uniform buffers (if dirty):

   - Write updated frame/camera/model data into pools.

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

     - Select pipeline:

     - Fetch from the pipeline cache or create if needed.

     - Bind vertex/index buffers and resource bind groups.

     - Issue draw calls via `RenderPass::draw` / `draw_indexed`.

### 7.3 Render Ordering & Batching (Per Camera)

- Opaque/masked objects are sorted by `(material_id, geometry_id)` to reduce
  state changes and batch draw calls.
- Transparent objects are sorted by depth for correct blending.

Draw calls are batched by runs of `(material_id, geometry_id)` after sorting.

3. Submit the frame to the surface swapchain.

---

## 8. Event System

The input layer aggregates events via the active platform proxy:

- Keyboard/pointer/touch from `winit` (desktop) or DOM (browser)
- Gamepad from `gilrs` (desktop) or the Web Gamepad API (browser)
- Window events (resize, close, focus, etc.) from the platform

These are translated into internal `EngineEvent` enums and pushed into
`event_queue`.

On `vulfram_receive_events`, the core:

1. Serializes `event_queue` into MessagePack (using `rmp-serde`).
2. Allocates a buffer, copies bytes, and exposes pointer & length.
3. Clears `event_queue` for the next frame.

---

## 9. Profiling Data

`ProfilingData` tracks these metrics:

- Timings (microseconds):
  - `commandProcessingUs`
  - `gamepadProcessingUs`
  - `eventLoopPumpUs`
  - `requestRedrawUs`
  - `serializationUs`
  - `renderTotalUs`
  - `renderShadowUs`
  - `renderWindowsUs`
  - `frameDeltaUs`
- Derived:
  - `fpsInstant`
- Per-window:
  - `windowFps[]` with `windowId`, `fpsInstant`, `frameDeltaUs`
- Counters:
  - `totalEventsDispatched`
  - `totalEventsCached`

On `vulfram_get_profiling`, the core:

1. Takes a snapshot of `ProfilingData`.
2. Serializes it into MessagePack.
3. Allocates and exposes the buffer via pointer & length.
