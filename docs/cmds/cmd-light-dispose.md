# CmdLightDispose

Removes a light source.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| windowId | u32  | ID of the window          |
| lightId  | u32  | ID of the light to remove |

## Response

Returns `CmdResultLightDispose`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was removed |
| message | String | Status or error message       |
