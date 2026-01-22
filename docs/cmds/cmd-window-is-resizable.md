# CmdWindowIsResizable

Checks if a window is resizable.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowIsResizable`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the check was successful |
| message | String | Status or error message          |
| content | bool   | Whether the window is resizable  |
