# CmdWindowCreate

Creates a new window and initializes its WGPU surface.

In web/WASM mode, the window is backed by a DOM canvas. Use `canvasId` to
attach to an existing `<canvas>` element.

## Arguments

| Field        | Type              | Description                                                              |
| ------------ | ----------------- | ------------------------------------------------------------------------ |
| windowId     | u32               | Unique ID for the new window                                             |
| title        | String            | (Optional) Window title (default: "")                                    |
| size         | UVec2             | (Optional) Initial size (default: 800x600)                               |
| position     | IVec2             | (Optional) Initial position (default: 0,0)                               |
| canvasId     | Option<String>    | DOM canvas id (required for web/WASM)                                    |
| borderless   | bool              | (Optional) Whether to hide decorations (default: false)                  |
| resizable    | bool              | (Optional) Whether the window can be resized (default: false)            |
| transparent  | bool              | (Optional) Whether the window background is transparent (default: false) |
| initialState | EngineWindowState | (Optional) Initial state (default: "windowed")                           |

Initial State mapping: "minimized", "maximized", "windowed", "fullscreen", "windowed-fullscreen".

## Response

Returns `CmdResultWindowCreate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the window was created |
| message | String | Status or error message        |
