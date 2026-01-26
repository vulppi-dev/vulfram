# CmdWindowRequestAttention

Requests user attention for a window (e.g., flashing the taskbar icon).

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field         | Type              | Description                                     |
| ------------- | ----------------- | ----------------------------------------------- |
| windowId      | u32               | ID of the window                                |
| attentionType | Option<UserAttentionType> | (Optional) Type of attention ("critical", "informational") |

## Response

Returns `CmdResultWindowRequestAttention`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the request was sent |
| message | String | Status or error message      |
