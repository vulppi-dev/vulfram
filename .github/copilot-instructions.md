# Instructions Memory

This file contains instructions for you to help it generate code that is consistent with the project's coding style, conventions, and requirements. It serves as a reference for Copilot to understand the context and expectations of the codebase.

**IMPORTANT**: Edit this file always you need remember specific pattern or style about this project.

---

# Memory Entries

## Communication & Documentation

### Language Standards

- **Communication with user**: Always in Brazilian Portuguese (pt-BR)
- **Code**: Always in English (variables, functions, types, comments)
- **Documentation**: Always in English (README, JSDoc, comments)

### Code Comments

- **Rust code**: Minimal comments - function and variable names should be self-descriptive
- **TypeScript bindings**: Complete JSDoc with:
  - Purpose explanation
  - Usage examples
  - NO need to define parameters/return types (TypeScript handles this)

## Project Information

### About Vulfram

- **Company**: Vulppi
- **Project name**: Vulfram (combination of Vulppi + Frame)
- **Name origin**: Vulppi comes from "Vulpes" (scientific name for fox)
- **Mascot**: Vulfi (The Frame Fox) - a futuristic, neon-tech fox with purple/magenta colors
- **Purpose**: Host-agnostic rendering and systems core exposed as a dynamic library
- **Design philosophy**:
  - Host controls the engine via small set of C-ABI functions
  - Core remains a black box owning windowing, input, GPU resources, and render pipeline
  - Binary communication via MessagePack for speed and compact format

### Technology Stack

- **Core**: Rust with WGPU (WebGPU implementation)
- **Windowing**: Winit for cross-platform window creation and OS events
- **Input**: Gilrs for gamepad support
- **Image processing**: `image` crate for texture decoding (PNG, JPEG, etc.)
- **Math**: `glam` for vectors/matrices, `bytemuck` for safe binary packing
- **Binding**: Multiple targets via C-ABI:
  - N-API for Node.js/Bun compatibility
  - `mlua` for Lua bindings
  - `PyO3` for Python bindings
  - Any FFI-capable environment
- **Serialization**: MessagePack via `serde` + `rmp-serde` for command/event communication
- **Language bindings**: TypeScript/JavaScript, Lua, Python

## Architecture Patterns

### Core Concepts

- **Host**: External program/runtime calling `vulfram_*` functions (Node.js, Bun, Lua, Python, etc.)
  - Manages game logic and world state
  - Generates logical IDs (entities, resources, buffers)
  - Builds MessagePack command batches
  - Drives main loop with `vulfram_tick(time, deltaTime)`
- **Core**: Rust dynamic library implementing Vulfram

  - Abstracts window, input, and rendering systems
  - Manages GPU resources via WGPU
  - Translates host commands into internal state changes
  - Never exposes internal Rust types, WGPU details, or Winit specifics

- **ABI**: C-ABI interface with primitive types only (`u32`, pointers, `size_t`, `double`)
  - Function return: `u32` status code (`0` = success, non-zero = error)
  - All functions are main-thread only, no concurrent calls
  - Thread safety must be implemented host-side

### Command/Event System

- **Commands**: Sent from Host → Core via `vulfram_send_queue(buffer, length)`
  - MessagePack-serialized batches
  - Create/update/destroy resources and components
  - Maintenance commands (e.g., `DiscardUnusedUploads`)
- **Messages**: Received Core → Host via `vulfram_receive_queue(out_ptr, out_length)`
  - Acknowledgments, error info, internal notifications
  - Host must copy bytes and free core-allocated buffer
- **Events**: Received Core → Host via `vulfram_receive_events(out_ptr, out_length)`

  - Input events (keyboard, mouse, gamepad, touch)
  - Window events (resize, close, focus, etc.)
  - Same buffer management as messages

- **Profiling**: Retrieved via `vulfram_profiling(out_ptr, out_length)`
  - Timing data for internal sections
  - Counters (draw calls, instances, resource counts)
  - Optional, called on-demand or per-frame for debug

### Components vs Resources

- **Components**: High-level structures attached to entities
  - Examples: `CameraComponent`, `ModelComponent`, `LightComponent` (future)
  - Always associated with host-generated `ComponentId`
  - Contain static data (colors, matrices) and/or references to resources
  - Created/updated via commands in `send_queue`
- **Resources**: Reusable data assets
  - **Sharable resources**: Referenced by logical IDs across multiple components
    - `ShaderResource` (`ShaderId`)
    - `GeometryResource` (`GeometryId`)
    - `MaterialResource` (`MaterialId`)
    - `TextureResource` (`TextureId`)
    - `SamplerResource` (future)
  - **Static resources**: Live inside specific components only
    - No separate logical ID
    - Examples: camera viewport, per-instance colors

### Type Organization

- **Commands**: Host-defined interfaces for operations
- **Events**: Core-defined structures for notifications
- **Results**: Status codes returned from C-ABI functions (`VulframResult` enum)
- **Enums**: Shared between Rust and bindings (snake_case in Rust → kebab-case in TS)

### Naming Conventions

- **Rust**: snake_case for functions/variables, PascalCase for types/structs
- **TypeScript**: camelCase for functions/variables, PascalCase for types/interfaces
- **C-ABI functions**: `vulfram_*` prefix (e.g., `vulfram_init`, `vulfram_tick`, `vulfram_send_queue`)
- **Commands**: Descriptive names in MessagePack (e.g., `CreateShader`, `UpdateCamera`)
- **Events**: Descriptive names for input/window notifications
- **Logical IDs**: Host-managed opaque integers
  - `ComponentId`, `ShaderId`, `GeometryId`, `MaterialId`, `TextureId`, `BufferId`
- **Internal handles**: Core-only references (never exposed to host)
  - `ShaderModuleHandle`, `RenderPipelineHandle`, `BufferHandle`, `TextureHandle`
  - `CameraInstanceHandle`, `MeshInstanceHandle`

### Code Structure

- **Rust modules**: Organized by feature (core/cmd, core/render, core/cache, etc.)
- **TypeScript**: Barrel exports via `index.ts` files
- **Core structure**:
  - `core/`: Engine core functionality
    - `buffers.rs`: Upload/download buffer management
    - `handler.rs`: Event and command handling
    - `lifecycle.rs`: Init/dispose lifecycle
    - `queue.rs`: Message/event queue management
    - `state.rs`: Central `EngineState` structure
    - `tick.rs`: Frame advancement logic
    - `cache/`: Event filtering and caching
    - `cmd/`: Command processing
    - `render/`: WGPU rendering system
  - `lib.rs`: FFI bindings entry point (N-API, mlua, PyO3)
- **MARK comments**: Use `// MARK: Section Name` for organization
- **Documentation**: Reference docs in `docs/` folder
  - `OVERVIEW.md`: High-level design and concepts
  - `ABI.md`: C-ABI function reference
  - `ARCH.md`: Architecture and lifecycle
  - `API.md`: Internal Rust implementation details
  - `GLOSSARY.md`: Terminology reference
  - `UI.md`: Visual identity and color palette
  - `MASCOT-DEFINITION.md`: Vulfi character definition

## Build & Development

### Commands

```bash
# Build native binding (run from project root)
bun run build:napi

# Development/testing (run from binding folder)
cd bind/ts
bun run dev
```

### Project Structure

- **Root**: Multi-workspace project
- **core/**: Rust crate (main implementation)
  - `Cargo.toml`: Rust dependencies and configuration
  - `src/`: Rust source code
    - `lib.rs`: FFI bindings entry point
    - `core/`: Engine implementation
- **bind/ts/**: TypeScript bindings
  - `src/`: TypeScript source
  - `napi/`: Generated native module
- **bind/lua/**: Lua bindings (mlua)
- **bind/python/**: Python bindings (PyO3)
- **docs/**: Complete documentation set
- **assets/**: Brand assets and resources

## Technical Decisions

### Serialization Format

- **Choice**: MessagePack (efficient binary serialization)
- **Reason**: Compact binary format with excellent performance and wide language support
- **Usage**: Command/Event communication between JS/TS and Rust
- **Libraries**:
  - Rust: `rmp-serde` (MessagePack with Serde support)
  - TypeScript: `msgpackr` (fast MessagePack implementation)

### Event Loop Architecture

- **Pattern**: External event loop (controlled from host)
- **Tick function**: `vulfram_tick(time, deltaTime)` advances engine
- **Events**: Pulled via `vulfram_receive_events()` each frame
- **Commands**: Pushed via `vulfram_send_queue()` as needed
- **Main loop flow**:
  1. Update host-side logic
  2. Perform uploads (optional)
  3. Send command batch
  4. Call `vulfram_tick`
  5. Receive messages
  6. Receive events
  7. Receive profiling (optional)

### Buffer Management

- **Upload**: `vulfram_upload_buffer(id, type, buffer, length)` - Send data to GPU
  - Host-generated `BufferId` for tracking
  - Types: shader source, vertex data, index data, texture image, etc.
  - One-shot semantics: consumed by `Create*` commands
- **Download**: `vulfram_download_buffer(id, type, out_ptr, out_length)` - Retrieve data from GPU
  - Frame captures, debug dumps, generated assets
- **Memory management**: Host must copy and free core-allocated buffers
- **Format**: Raw byte arrays (Uint8Array in JS/TS)
- **Cleanup**: `DiscardUnusedUploads` command removes unconsumed uploads

### Window Management

- **Multi-window**: Supported with unique window IDs
- **Lifecycle**: Create → Use → Close Request → Explicit Close
- **Events**: Window events include window ID for identification
- **State**: Normal, Minimized, Maximized, Fullscreen

### Input Handling

- **Unified pointer**: Mouse, touch, and pen through PointerEvent
- **Physical keys**: KeyCode based on physical location (layout-independent)
- **Gamepad**: Standard mapping with automatic detection
- **Modifiers**: Tracked separately from key events

## Dependencies

### Rust Crates

- `wgpu`: GPU abstraction layer
- `winit`: Cross-platform windowing
- `gilrs`: Gamepad input
- `image`: Texture file decoding (PNG, JPEG, etc.)
- `glam`: Vector, matrix, quaternion types
- `bytemuck`: Safe binary packing for GPU upload
- `napi`: Node.js N-API bindings
- `mlua`: Lua bindings
- `PyO3`: Python bindings
- `serde`: Serialization framework
- `serde_repr`: Numeric enum serialization
- `rmp-serde`: MessagePack serialization with Serde support

### TypeScript/JavaScript

- `msgpackr`: MessagePack encoding/decoding
- `bun`: Primary runtime (but Node.js compatible)

## Best Practices

### Error Handling

- Always return `VulframResult` enum from core functions
- Results include success/failure status and optional message
- Command results are typed per command type

### Resource Management

- Explicit lifecycle management (init → use → dispose)
- Buffer IDs managed by user code
- Window IDs returned on creation
- Cleanup on dispose

### Performance Considerations

- Batch commands when possible
- Use buffer upload/download efficiently
- Process events in bulk each frame
- MessagePack serialization for speed and compact binary format

### Event Optimization

- **Cache system**: Events are filtered to avoid dispatching duplicates
- **Window events**: Cache position, size, scale factor, focus, occluded, theme
- **Pointer events**: 1px threshold for position changes
- **Gamepad events**:
  - Dead zone: 0.1 for analog sticks/triggers
  - Change threshold: 0.01 for axes, 0.05 for button values
  - Button state changes cached per gamepad
- **Keyboard events**: Modifier state cached to avoid duplicate events
- **Cache structure**: `src/core/cache/` with managers for window, input, and gamepad state

### Redraw Optimization

- **Dirty flag system**: Windows only request redraw when marked as dirty
- **Selective rendering**: Only visible windows are rendered
- **Needs redraw flag**: Global flag to skip redraw request if nothing changed
- **Automatic marking**: Window resize and surface reconfiguration automatically mark dirty
- **Manual marking**: `mark_window_dirty()` and `mark_all_windows_dirty()` for explicit control

### Type Safety

- Full TypeScript types for all commands/events
- Discriminated unions for command/event types
- Rust type safety with strong typing
- Shared enums between Rust and TypeScript
- **Numeric enums**: Always use `serde_repr` with `u32` representation for enums that can be numeric
  - Add `use serde_repr::{Deserialize_repr, Serialize_repr};`
  - Use `#[derive(Serialize_repr, Deserialize_repr)]`
  - Add `#[repr(u32)]` attribute
  - Assign explicit numeric values to variants
  - **CRITICAL**: TypeScript enums MUST match Rust numeric values exactly for MessagePack serialization
  - TypeScript enums declared in `enums.ts` with same numeric values as Rust
  - String union types are only acceptable for non-serialized types (e.g., helper functions)

### Enum Synchronization (Rust ↔ TypeScript)

All enums serialized via MessagePack MUST be numeric and synchronized:

- **Rust side**:
  - Use `#[derive(Serialize_repr, Deserialize_repr)]`
  - Use `#[repr(u32)]`
  - Explicit numeric values (e.g., `Float = 0, Int = 1`)
  
- **TypeScript side**:
  - Declare as numeric enum in `bind/ts/src/enums.ts`
  - Match exact numeric values from Rust
  - Import and use enum types (not string literals)
  
- **Serialized enums** (MUST be numeric):
  - `UniformType`: Float=0, Int=1, UInt=2, Bool=3, Vec2=4...Mat4x4=21, AtomicInt=22, AtomicUInt=23
  - `TextureSampleType`: Float=0, Depth=1, Sint=2, Uint=3
  - `TextureViewDimension`: D1=0, D2=1, D2Array=2, Cube=3, CubeArray=4, D3=5
  - `SamplerBindingType`: Filtering=0, NonFiltering=1, Comparison=2
  - `TextureFormat`: R8Unorm=0, R8Snorm=1...Depth32FloatStencil8=37
  - `VertexFormat`: Uint8x2=0...Float64x4=33
  - `IndexFormat`: Uint16=0, Uint32=1
  
- **Non-serialized types** (can be string unions):
  - `VertexSemantic`: 'position' | 'normal' | 'uv0' | 'color0' | etc. (used only in TypeScript)
  - `ViewportMode`: 'relative' | 'absolute' (used only in TypeScript)

### Layer Masking and Visibility

- **LayerMask system**: `u32` bitmasks for visibility control
- **Camera layers**: Each camera has `layerMaskCamera`
- **Component layers**: Each model has `layerMaskComponent`
- **Visibility rule**: `(layerMaskCamera & layerMaskComponent) > 0`
- **Use cases**:
  - Separate world/UI rendering
  - Team-based filtering
  - Debug-only geometry
  - Special render passes (picking, shadows, etc.)

### Dynamic Offset System

- **Uniform buffer offsets**: Always dynamic (not embedded in bind groups)
- **has_dynamic_offset**: Always set to `true` for all uniform buffer bindings
- **Bind groups**: Created with `offset: 0` and `size: None`
- **Draw time offsets**: Passed via `set_bind_group()` with offset array
- **Group continuity**: Bind group layout indices must be contiguous (0, 1, 2...)
  - wgpu requires all bind group slots to be filled in the pipeline layout
  - If shader uses `@group(0)` and `@group(2)`, must create layouts for 0, 1, and 2
  - Empty layouts (with no entries) are created for unused intermediate groups
- **Group mapping**: Layout index = group number (direct mapping after ensuring continuity)
- **Benefits**: Single bind group per shader group shared across all components using dynamic offsets
