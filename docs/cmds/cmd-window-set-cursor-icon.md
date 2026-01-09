# CmdWindowSetCursorIcon

Sets the system cursor icon for a window.

## Arguments

| Field    | Type       | Description                                                 |
| -------- | ---------- | ----------------------------------------------------------- |
| windowId | u32        | ID of the window                                            |
| icon     | CursorIcon | ID of the cursor icon (Default=0, Pointer=3, Hand=16, etc.) |

## Response

Returns `CmdResultWindowSetCursorIcon`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the icon was set |
| message | String | Status or error message  |
