# CmdWindowSetDecorations

Enables or disables window decorations (borders, title bar).

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field       | Type | Description                 |
| ----------- | ---- | --------------------------- |
| windowId    | u32  | ID of the window            |
| decorations | bool | Whether to show decorations |

## Response

Returns `CmdResultWindowSetDecorations`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the decorations were set |
| message | String | Status or error message          |
