# CmdWindowSetCursorIcon

Sets the system cursor icon for a window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type       | Description                                                 |
| -------- | ---------- | ----------------------------------------------------------- |
| windowId | u32        | ID of the window                                            |
| icon     | CursorIcon | Cursor icon enum value                                      |

### CursorIcon (Enum)

- `Default` = 0
- `ContextMenu` = 1
- `Help` = 2
- `Pointer` = 3
- `Progress` = 4
- `Wait` = 5
- `Cell` = 6
- `Crosshair` = 7
- `Text` = 8
- `VerticalText` = 9
- `Alias` = 10
- `Copy` = 11
- `Move` = 12
- `NoDrop` = 13
- `NotAllowed` = 14
- `Grab` = 15
- `Grabbing` = 16
- `EResize` = 17
- `NResize` = 18
- `NeResize` = 19
- `NwResize` = 20
- `SResize` = 21
- `SeResize` = 22
- `SwResize` = 23
- `WResize` = 24
- `EwResize` = 25
- `NsResize` = 26
- `NeswResize` = 27
- `NwseResize` = 28
- `ColResize` = 29
- `RowResize` = 30
- `AllScroll` = 31
- `ZoomIn` = 32
- `ZoomOut` = 33

## Response

Returns `CmdResultWindowSetCursorIcon`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the icon was set |
| message | String | Status or error message  |
