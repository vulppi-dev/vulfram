# CmdWindowSetResizable

Enables or disables the ability to resize a window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field     | Type | Description                            |
| --------- | ---- | -------------------------------------- |
| windowId  | u32  | ID of the window                       |
| resizable | bool | Whether the window should be resizable |

## Response

Returns `CmdResultWindowSetResizable`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the property was set |
| message | String | Status or error message      |
