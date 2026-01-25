# 📚 Vulfram Documentation Index

Welcome to the Vulfram documentation! This index will guide you to the right documents based on your needs.

---

## 🎯 Quick Navigation

### I want to...

- **Understand Vulfram's design** → Start with [OVERVIEW.md](OVERVIEW.md)
- **Create a language binding** → Read [ABI.md](ABI.md) and [ARCH.md](ARCH.md)
- **Contribute to the Rust core** → Check [API.md](API.md) and [GLOSSARY.md](GLOSSARY.md)
- **Define a render graph** → Read [RENDER-GRAPH.md](RENDER-GRAPH.md)
- **Learn terminology** → See [GLOSSARY.md](GLOSSARY.md)
- **Understand the architecture** → Read [ARCH.md](ARCH.md)
- **Understand platform separation** → Read [PLATFORM-PROXIES.md](PLATFORM-PROXIES.md)

---

## 📖 Documentation Structure

### For Engine Users (Binding Authors & Integrators)

If you're building a language binding (Node.js, Lua, Python, etc.) or integrating Vulfram into your application:

1. **[OVERVIEW.md](OVERVIEW.md)** - _Start here!_

   - Design goals and philosophy
   - High-level architecture
   - Components vs Resources
   - IDs and handles
   - Layer masking system

2. **[ABI.md](ABI.md)** - _Essential for bindings_

   - C-ABI function reference
   - Calling conventions
   - Error handling
   - MessagePack serialization
   - Buffer management (upload)

3. **[ARCH.md](ARCH.md)** - _Implementation patterns_

   - Lifecycle management
   - Main loop structure
   - Recommended frame flow
   - Host responsibilities
   - Upload system

4. **[cmds/](cmds/)** - _Command Reference_

   - Detailed documentation for every engine command (Window, Camera, Model, Light, etc.)

5. **[RENDER-GRAPH.md](RENDER-GRAPH.md)** - _Host-defined render graph_

6. **[GLOSSARY.md](GLOSSARY.md)** - _Terminology reference_
   - Core concepts (Host, Core, ABI)
   - Components vs Resources
   - IDs and handles
   - Naming conventions

---

### For Core Contributors (Rust Developers)

If you're working on the Rust core implementation:

1. **[OVERVIEW.md](OVERVIEW.md)** - _Foundational concepts_

   - Design principles
   - Architecture overview
   - Component and resource model

2. **[ARCH.md](ARCH.md)** - _System architecture_

   - Component lifecycle
   - Resource management
   - Visibility and layer masking

3. **[API.md](API.md)** - _Internal implementation_

   - Crate dependencies
   - Engine state structure
   - Resource and component tables
   - Command flow
   - Upload handling
   - Rendering system
   - Event system
   - Profiling data

4. **[GLOSSARY.md](GLOSSARY.md)** - _Internal terminology_
   - Function naming patterns
   - Internal handles
   - Queue types
   - File organization

---

## 🔍 Document Details

### [OVERVIEW.md](OVERVIEW.md)

**Purpose:** High-level introduction to Vulfram  
**Audience:** Everyone (start here)  
**Topics:**

- Design goals (host-agnostic, minimal surface, binary communication)
- Architecture diagram (Host → Core → GPU)
- Components vs Resources
- Logical IDs and internal handles
- One-shot upload system
- Layer masking for visibility
- What the host sees vs. what it doesn't
- Asynchronous resource linking (fallback-driven)

### [ABI.md](ABI.md)

**Purpose:** C-ABI specification and usage contract  
**Audience:** Binding authors, advanced users  
**Topics:**

- Return codes (`VulframResult`)
- Threading and reentrancy rules
- MessagePack serialization
- Output buffer management
- Function reference:
  - `vulfram_init()` / `vulfram_dispose()`
  - `vulfram_send_queue()`
  - `vulfram_receive_queue()`
  - `vulfram_receive_events()`
  - `vulfram_upload_buffer()`
  - `vulfram_tick()`
  - `vulfram_get_profiling()`
- Recommended frame loop
- Error handling guidelines

### [ARCH.md](ARCH.md)

**Purpose:** Architecture, lifecycle, and main loop patterns  
**Audience:** Binding authors, integrators  
**Topics:**

- High-level architecture
- Host vs Core responsibilities
- Components, Resources, and Instances
- Layer masking and visibility
- Core lifecycle (startup, loading, main loop, shutdown)
- Recommended main loop structure
- One-shot upload pattern and cleanup

### [API.md](API.md)

**Purpose:** Internal Rust API documentation  
**Audience:** Core contributors (Rust developers)  
**Topics:**

- Crate dependencies (winit, wgpu, gilrs, serde, etc.)
- Engine state structure
- Resource management (Geometries, Textures, Materials)
- Component instances (Cameras, Meshes)
- Internal command flow
- Upload table handling
- Rendering system (buffers, pipelines, render passes)
- Event system integration
- Profiling data collection

### [GLOSSARY.md](GLOSSARY.md)

**Purpose:** Terminology and naming conventions  
**Audience:** Everyone (reference guide)  
**Topics:**

- Core concepts (Host, Core, ABI)
- Components vs Resources
- Logical IDs vs Internal Handles
- Upload and buffer terminology
- Queue types (command, response, event)
- Layer masking
- Function naming (`vulfram_*`)
- Documentation file structure

---

## 🎓 Learning Paths

### Path 1: Node.js Binding Developer (N-API)

```
OVERVIEW.md
    ↓
ABI.md (focus on N-API patterns)
    ↓
ARCH.md (main loop implementation)
    ↓
GLOSSARY.md (reference as needed)
```

### Path 2: Python/Lua Binding Developer

```
OVERVIEW.md
    ↓
ABI.md (focus on FFI patterns)
    ↓
ARCH.md (lifecycle and loop)
    ↓
GLOSSARY.md (reference as needed)
```

### Path 3: Rust Core Contributor

```
OVERVIEW.md
    ↓
ARCH.md
    ↓
API.md (deep dive into internals)
    ↓
GLOSSARY.md (naming conventions)
```

### Path 4: Game Developer (Using Existing Binding)

```
OVERVIEW.md (optional, for understanding)
    ↓
Language-specific binding documentation
    ↓
GLOSSARY.md (for terminology)
```

---

## 🔗 External Resources

- **[README.md](../README.md)** - Project overview and quick start
- **[MASCOT-DEFINITION.md](MASCOT-DEFINITION.md)** - Brand guidelines
- **[UI.md](UI.md)** - User interface guidelines
- **[PLATFORM-PROXIES.md](PLATFORM-PROXIES.md)** - Platform proxy architecture
- **[Copilot Instructions](../.github/copilot-instructions.md)** - Development patterns

---

## 📝 Document Conventions

### Code Examples

- **Conceptual**: Pseudo-code for clarity
- **Rust**: Actual or near-actual Rust code
- **C**: C-ABI function signatures
- **MessagePack**: Logical structure (not binary)

### Terminology

- **Host**: External program calling Vulfram
- **Core**: The Rust library (Vulfram itself)
- **Component**: Scene entity behavior (Camera, Model, etc.)
- **Resource**: Reusable asset (Geometry, Texture, Material, etc.)
- **Logical ID**: Integer ID visible to host
- **Handle**: Internal core reference (not exposed)

---

## 💡 Tips for Reading

1. **Start with OVERVIEW.md** - It provides essential context for everything else
2. **Use GLOSSARY.md** - Keep it open as a reference for terminology
3. **Focus on your role** - Follow the learning path that matches your needs
4. **Cross-reference** - Documents link to each other for deeper dives
5. **Check examples** - Code samples demonstrate concepts in practice

---

## 🤔 Need Help?

- **Found an error?** Open an issue on GitHub
- **Need clarification?** Check GLOSSARY.md or ask in discussions
- **Contributing?** See [CONTRIBUTING guidelines](../README.md#-contributing)

---

<div align="center">

**Happy coding with Vulfram! 🦊**

Made with ❤️ by the Vulppi team

</div>
