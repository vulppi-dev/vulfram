# CmdWindowRequestAttention

Requests user attention for a window (e.g., flashing the taskbar icon).

## Arguments

| Field         | Type              | Description                                     |
| ------------- | ----------------- | ----------------------------------------------- |
| windowId      | u32               | ID of the window                                |
| attentionType | UserAttentionType | Type of attention (Critical=0, Informational=1) |

## Response

Returns `CmdResultWindowRequestAttention`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the request was sent |
| message | String | Status or error message      |
