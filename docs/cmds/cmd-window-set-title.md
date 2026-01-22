# CmdWindowSetTitle

Sets the title of an existing window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type   | Description      |
| -------- | ------ | ---------------- |
| windowId | u32    | ID of the window |
| title    | String | New title        |

## Response

Returns `CmdResultWindowSetTitle`:

| Field   | Type   | Description               |
| ------- | ------ | ------------------------- |
| success | bool   | Whether the title was set |
| message | String | Status or error message   |
