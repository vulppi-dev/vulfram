# CmdWindowGetOuterSize

Retrieves the outer size of a window, including Decorations (borders, title bar).

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetOuterSize`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the size was retrieved |
| message | String | Status or error message        |
| content | UVec2  | Current outer size dimensions  |
