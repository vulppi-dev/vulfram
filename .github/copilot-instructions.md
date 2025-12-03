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
- **Purpose**: High-performance game engine powered by WebGPU

### Technology Stack

- **Core**: Rust with WGPU (WebGPU implementation)
- **Windowing**: Winit
- **Input**: Gilrs (gamepad support)
- **Binding**: N-API for Node.js/Bun compatibility
- **Serialization**: CBOR for efficient command/event communication
- **Language bindings**: TypeScript/JavaScript

## Architecture Patterns

### Command/Event System

- **Commands**: Sent from JS/TS → Rust via `vulframSendQueue()`
- **Events**: Received from Rust → JS/TS via `vulframReceiveQueue()`
- **Serialization**: CBOR format for performance
- **Pattern**: Queue-based architecture for async communication

### Type Organization

- **Commands**: `src/cmds/*.ts` - TypeScript command interfaces
- **Events**: `src/events/*.ts` - TypeScript event interfaces
- **Results**: Typed result objects with success/failure status
- **Enums**: Shared between Rust and TypeScript (snake_case → kebab-case)

### Naming Conventions

- **Rust**: snake_case for functions/variables, PascalCase for types/structs
- **TypeScript**: camelCase for functions/variables, PascalCase for types/interfaces
- **Commands**: Prefix with `Cmd` (e.g., `CmdWindowCreate`)
- **Events**: Suffix with `Event` (e.g., `WindowCreatedEvent`)
- **Command types**: `cmd-*` pattern (e.g., `'cmd-window-create'`)
- **Event types**: `on-*` pattern (e.g., `'on-create'`)

### Code Structure

- **Rust modules**: Organized by feature (core/cmd, core/render, etc.)
- **TypeScript**: Barrel exports via `index.ts` files
- **MARK comments**: Use `// MARK: Section Name` for organization in TypeScript
- **Rust sections**: Use `// MARK: Section Name` or module structure

## Build & Development

### Commands

```bash
# Build native binding (run from project root)
bun run build:napi

# Development/testing (run from binding folder)
cd binding
bun run dev
```

### Project Structure

- **Root**: Rust crate with Cargo.toml
- **src/**: Rust source code
  - `core/`: Engine core functionality
  - `lib.rs`: N-API bindings entry point
- **binding/**: TypeScript bindings
  - `src/`: TypeScript source
  - `napi/`: Generated native module
- **assets/**: Brand assets and resources

## Technical Decisions

### Serialization Format

- **Choice**: CBOR (Concise Binary Object Representation)
- **Reason**: More efficient than JSON for binary data transfer
- **Usage**: Command/Event communication between JS/TS and Rust

### Event Loop Architecture

- **Pattern**: External event loop (controlled from JS/TS)
- **Tick function**: `vulframTick(time, deltaTime)` advances engine
- **Events**: Pulled via `vulframReceiveQueue()` each frame
- **Commands**: Pushed via `vulframSendQueue()` as needed

### Buffer Management

- **Upload**: `vulframUploadBuffer()` - Send data to GPU (textures, meshes)
- **Download**: `vulframDownloadBuffer()` - Retrieve data from GPU
- **Clear**: `vulframClearBuffer()` - Free GPU memory
- **Format**: Uint8Array for cross-platform compatibility

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
- `napi`: Node.js N-API bindings
- `serde`: Serialization framework (with CBOR support via ciborium)

### TypeScript/JavaScript

- `cbor2`: CBOR encoding/decoding
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
- CBOR serialization for speed

### Type Safety

- Full TypeScript types for all commands/events
- Discriminated unions for command/event types
- Rust type safety with strong typing
- Shared enums between Rust and TypeScript
