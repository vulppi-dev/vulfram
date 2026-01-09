# CmdWindowCreate

Creates a new window and initializes its WGPU surface.

## Arguments

| Field        | Type              | Description                                                                              |
| ------------ | ----------------- | ---------------------------------------------------------------------------------------- |
| windowId     | u32               | Unique ID for the new window                                                             |
| title        | String            | Window title                                                                             |
| size         | UVec2             | Initial size (default: 800x600)                                                          |
| position     | IVec2             | Initial position                                                                         |
| borderless   | bool              | Whether to hide decorations                                                              |
| resizable    | bool              | Whether the window can be resized                                                        |
| transparent  | bool              | Whether the window background is transparent                                             |
| initialState | EngineWindowState | Initial state (Minimized=0, Maximized=1, Windowed=2, Fullscreen=3, WindowedFullscreen=4) |

## Response

Returns `CmdResultWindowCreate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the window was created |
| message | String | Status or error message        |
