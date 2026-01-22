# CmdWindowGetSize

Retrieves the inner size (drawable area) of an existing window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetSize`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the size was retrieved |
| message | String | Status or error message        |
| content | UVec2  | Current inner size dimensions  |
