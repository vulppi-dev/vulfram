# CmdWindowGetSize

Retrieves the inner size (drawable area) of an existing window.

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
