# CmdWindowSetPosition

Sets the position of an existing window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type  | Description              |
| -------- | ----- | ------------------------ |
| windowId | u32   | ID of the window         |
| position | IVec2 | New position coordinates |

## Response

Returns `CmdResultWindowSetPosition`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the position was set |
| message | String | Status or error message      |
