# CmdWindowFocus

Requests that a window be focused.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowFocus`:

| Field   | Type   | Description                        |
| ------- | ------ | ---------------------------------- |
| success | bool   | Whether the focus request was sent |
| message | String | Status or error message            |
