# CmdWindowClose

Closes an existing window and cleans up its resources.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| windowId | u32  | ID of the window to close |

## Response

Returns `CmdResultWindowClose`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the window was closed |
| message | String | Status or error message       |
