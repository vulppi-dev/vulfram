# CmdWindowSetCursorVisible

Shows or hides the mouse cursor when it's over the window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type | Description                          |
| -------- | ---- | ------------------------------------ |
| windowId | u32  | ID of the window                     |
| visible  | bool | Whether the cursor should be visible |

## Response

Returns `CmdResultWindowSetCursorVisible`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the visibility was set |
| message | String | Status or error message        |
