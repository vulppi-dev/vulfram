# CmdWindowGetState

Retrieves the current state of a window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultWindowGetState`:

| Field   | Type              | Description                     |
| ------- | ----------------- | ------------------------------- |
| success | bool              | Whether the state was retrieved |
| message | String            | Status or error message         |
| content | EngineWindowState | Current window state            |
