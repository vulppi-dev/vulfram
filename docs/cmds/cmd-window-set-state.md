# CmdWindowSetState

Sets the state of a window (Minimized, Maximized, Fullscreen, etc.).

## Arguments

| Field    | Type              | Description                                                                             |
| -------- | ----------------- | --------------------------------------------------------------------------------------- |
| windowId | u32               | ID of the window                                                                        |
| state    | EngineWindowState | Target state (Minimized=0, Maximized=1, Windowed=2, Fullscreen=3, WindowedFullscreen=4) |

## Response

Returns `CmdResultWindowSetState`:

| Field   | Type   | Description               |
| ------- | ------ | ------------------------- |
| success | bool   | Whether the state was set |
| message | String | Status or error message   |
