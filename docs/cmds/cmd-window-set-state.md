# CmdWindowSetState

Sets the state of a window ("minimized", "maximized", "fullscreen", etc.).

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type              | Description                                                                             |
| -------- | ----------------- | --------------------------------------------------------------------------------------- |
| windowId | u32               | ID of the window                                                                        |
| state    | EngineWindowState | Target state ("minimized", "maximized", "windowed", "fullscreen", "windowed-fullscreen") |

## Response

Returns `CmdResultWindowSetState`:

| Field   | Type   | Description               |
| ------- | ------ | ------------------------- |
| success | bool   | Whether the state was set |
| message | String | Status or error message   |
