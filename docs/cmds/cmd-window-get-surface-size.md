# CmdWindowGetSurfaceSize

Retrieves the actual size of the rendering surface.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetSurfaceSize`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the size was retrieved  |
| message | String | Status or error message         |
| content | UVec2  | Current surface size dimensions |
