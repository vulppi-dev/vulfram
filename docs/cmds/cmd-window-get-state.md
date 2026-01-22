# CmdWindowGetState

Retrieves the current state of a window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetState`:

| Field   | Type              | Description                     |
| ------- | ----------------- | ------------------------------- |
| success | bool              | Whether the state was retrieved |
| message | String            | Status or error message         |
| content | EngineWindowState | Current window state            |
