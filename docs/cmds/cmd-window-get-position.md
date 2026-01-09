# CmdWindowGetPosition

Retrieves the position of an existing window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetPosition`:

| Field   | Type   | Description                        |
| ------- | ------ | ---------------------------------- |
| success | bool   | Whether the position was retrieved |
| message | String | Status or error message            |
| content | IVec2  | Current position coordinates       |
