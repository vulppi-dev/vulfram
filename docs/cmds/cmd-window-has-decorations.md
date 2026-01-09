# CmdWindowHasDecorations

Checks if window decorations are enabled.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowHasDecorations`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the check was successful |
| message | String | Status or error message          |
| content | bool   | Whether decorations are enabled  |
