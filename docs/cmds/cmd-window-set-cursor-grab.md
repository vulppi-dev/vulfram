# CmdWindowSetCursorGrab

Configures mouse cursor grabbing/locking.

## Arguments

| Field    | Type           | Description                              |
| -------- | -------------- | ---------------------------------------- |
| windowId | u32            | ID of the window                         |
| mode     | CursorGrabMode | Grab mode (None=0, Confined=1, Locked=2) |

## Response

Returns `CmdResultWindowSetCursorGrab`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the grab mode was set |
| message | String | Status or error message       |
