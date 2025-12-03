<div align="center">
  <img src="./assets/brand.svg" alt="Vulfram" width="400" />
  
  # Vulfram
  
  **A High-Performance Game Engine powered by WebGPU**
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.md)
  [![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
  [![TypeScript](https://img.shields.io/badge/typescript-5.0+-blue.svg)](https://www.typescriptlang.org/)
</div>

---

## 🦊 About Vulfram

**Vulfram** is a modern, high-performance game engine developed by **Vulppi**. The name combines "Vulppi" (derived from _Vulpes_, the scientific name for fox) and "Frame", representing our mission to create perfect frames for incredible interactive experiences.

Built on **Rust** and **WebGPU**, Vulfram offers:

- 🚀 **High Performance**: GPU-accelerated rendering with WGPU
- 🔄 **Cross-Platform**: Native support for Windows, macOS, and Linux
- 🎮 **Complete Input System**: Keyboard, mouse, touch, and gamepads
- 🪟 **Advanced Window Management**: Full control over multiple windows
- 🔌 **TypeScript/JavaScript Bindings**: Modern interface via N-API
- ⚡ **Command/Event Architecture**: Efficient queue-based communication

## 🏗️ Architecture

Vulfram uses a command and event-based architecture that enables efficient communication between JavaScript/TypeScript and the Rust core:

```
┌─────────────────────────────────────┐
│   JavaScript/TypeScript Layer       │
│   (Bun, Node.js, Electron, etc.)    │
└─────────────┬───────────────────────┘
              │ Commands (CBOR)
              ├─► vulframSendQueue()
              │
              │ Events (CBOR)
              ├─► vulframReceiveQueue()
              │
┌─────────────▼───────────────────────┐
│        Rust Core (WGPU)             │
│  • Window Management (Winit)        │
│  • GPU Rendering (WGPU)             │
│  • Input Processing (Gilrs)         │
│  • Event Loop                       │
└─────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Node.js** 18+ or **Bun** 1.0+
- **Vulkan**, **Metal**, or **DirectX 12** updated drivers

### Installation

```bash
# Clone the repository
git clone https://github.com/vulppi-dev/vulfram.git
cd vulfram

# Install dependencies
bun install

# Build native binding
bun run build:napi
```

### Basic Example

```typescript
import {
  vulframInit,
  vulframSendQueue,
  vulframReceiveQueue,
  vulframTick,
  vulframDispose,
  VulframResult,
  type EngineBatchCmds,
} from '@vulppi/vulfram';

// Initialize the engine
if (vulframInit() !== VulframResult.Success) {
  throw new Error('Failed to initialize Vulfram');
}

// Create a window
const createWindowCmd: EngineBatchCmds = {
  cmds: [
    {
      type: 'cmd-window-create',
      content: {
        title: 'My Game',
        size: { width: 1280, height: 720 },
        position: { x: 100, y: 100 },
        borderless: false,
        resizable: true,
        initialState: 'normal',
      },
    },
  ],
};

vulframSendQueue(createWindowCmd);

// Game loop
let lastTime = Date.now();
let running = true;

function gameLoop() {
  if (!running) return;

  // Calculate delta time
  const currentTime = Date.now();
  const deltaTime = currentTime - lastTime;
  lastTime = currentTime;

  // Process events
  const [events, result] = vulframReceiveQueue();
  if (result === VulframResult.Success) {
    for (const event of events.events) {
      if (
        event.kind === 'window' &&
        event.content.event === 'on-close-request'
      ) {
        running = false;
        break;
      }
    }
  }

  // Update the engine
  vulframTick(currentTime, deltaTime);

  // Next frame
  if (running) {
    setTimeout(gameLoop, 16); // ~60 FPS
  } else {
    vulframDispose();
  }
}

gameLoop();
```

## 📚 Main Features

### Window Management

- Creation and destruction of multiple windows
- Control of size, position, and state (minimized, maximized, fullscreen)
- Customizable decorations (borders, title bar)
- Customizable icons and cursor
- Drag-and-drop file support

### Input System

- **Keyboard**: Key events with modifier and IME support
- **Mouse/Touch**: Movement, buttons, scroll, and gestures (pinch, pan, rotate)
- **Gamepad**: Automatic detection, buttons, and analog sticks
- Unified input with `PointerEvent` for mouse/touch/pen

### Rendering

- GPU-accelerated via WebGPU (WGPU)
- Buffer upload/download for textures and meshes
- Efficient CPU-GPU synchronization

### Communication

- CBOR serialization for fast communication
- Command and event queues
- Typed result system

## 🛠️ Development

```bash
# Build Rust project
cargo build --release

# Build TypeScript binding (from project root)
bun run build:napi

# Development/testing (from binding folder)
cd binding
bun run dev

# Tests
cargo test
```

## 📦 Project Structure

```
vulfram/
├── src/                    # Rust code
│   ├── core/              # Engine core
│   │   ├── cmd/           # Command system
│   │   │   ├── events/    # Event processing
│   │   │   └── win/       # Window commands
│   │   └── render/        # Rendering system
│   └── lib.rs             # N-API binding entry point
├── binding/               # TypeScript bindings
│   └── src/
│       ├── cmds/          # Command types
│       ├── events/        # Event types
│       └── index.ts       # Public API
└── assets/                # Visual resources
```

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the project
2. Create a branch for your feature (`git checkout -b feature/MyFeature`)
3. Commit your changes (`git commit -m 'Add MyFeature'`)
4. Push to the branch (`git push origin feature/MyFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License. See the [LICENSE.md](LICENSE.md) file for details.

## 🦊 About Vulppi

**Vulppi** is a company focused on creating cutting-edge technologies for game development and interactive applications. Our name comes from _Vulpes_, the scientific name for fox, symbolizing agility, intelligence, and adaptability.

---

<div align="center">
  Made with ❤️ by the Vulppi team
</div>
