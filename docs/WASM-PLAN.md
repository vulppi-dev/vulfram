# WASM/Web Support - Phase 1 Audit

Scope: planning and dependency audit for a web/WASM build, without code changes
to runtime behavior yet.

## Targets and Build Modes

- Target: `wasm32-unknown-unknown`
- New feature: `wasm`
- Existing features: `ffi`, `napi`, `lua`, `python`
- Web mode uses WebGPU + DOM + Gamepad API, without `winit` or `gilrs`.

## Current Native Dependencies That Must Be Gated

Modules tied to `winit` (windowing + event loop + input):

- `src/core/lifecycle.rs` (EventLoop creation)
- `src/core/singleton.rs` (EventLoop, EventLoopProxy)
- `src/core/handler.rs` (ApplicationHandler and all WindowEvent mapping)
- `src/core/tick.rs` (EventLoopExtPumpEvents)
- `src/core/window/*` (Window types, cursor, focus, fullscreen, icons, etc.)
- `src/core/input/events/converters.rs` (winit key/mouse/touch conversions)
- `src/core/system/notification.rs` (EventLoopProxy usage)

Modules tied to `gilrs` (gamepad):

- `src/core/gamepad/*` (state, converters, events, processing)
- `src/core/tick.rs` (gilrs polling)

Modules that may need web-specific replacements:

- `env_logger`, `ctrlc`, `notify-rust` (not usable in wasm, must be cfg-gated)
- `pollster` (in wasm usually replaced by `wasm-bindgen-futures` or async init)

## Web API Mapping (Planned)

Window creation:

- `CmdWindowCreate` will accept `canvasId` in web mode.
- The "window" wraps a `HtmlCanvasElement` and surface is created from it.

Input events:

- DOM listeners mapped into existing `EngineEvent` types:
  - Keyboard: keydown/keyup/keypress -> `KeyboardEvent`
  - Pointer: mouse/pointer/touch -> `PointerEvent`
  - Wheel: `WheelEvent` -> `ScrollDelta`
  - Focus/blur/resize -> `WindowEvent` equivalents

Gamepad:

- Use `Navigator.get_gamepads()` for polling each `vulfram_tick`.
- Map to existing `GamepadEvent` with the same cache + deadzone filters.

## Rendering Considerations

- WGPU in wasm uses WebGPU backend; surface creation depends on canvas.
- Initialization may need to be async (device/adapter).
- Limits and formats can differ; must align with WebGPU constraints.

## Required Feature / cfg Strategy (Draft)

- `cfg(feature = "wasm")` for web-only code paths.
- `cfg(not(feature = "wasm"))` for native (winit/gilrs) code paths.
- `cfg(target_arch = "wasm32")` where needed for API differences.

## Command Contract Changes (Web)

- `CmdWindowCreate` will include an optional `canvasId` when `wasm` feature is
  enabled. In native mode, the field is ignored or absent.

## Open Questions

- Do we keep `ffi`/`napi`/`lua`/`python` compatible with `wasm`?
- How should surface resize be driven: DOM resize observer vs explicit command?
- Do we want a separate web entrypoint or keep shared ABI?
