# CmdWindowSetCursorIcon

Sets the system cursor icon for a window.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Arguments

| Field    | Type       | Description            |
| -------- | ---------- | ---------------------- |
| windowId | u32        | ID of the window       |
| icon     | CursorIcon | Cursor icon enum value |

### CursorIcon (Enum)

- `default`
- `context-menu`
- `help`
- `pointer`
- `progress`
- `wait`
- `cell`
- `crosshair`
- `text`
- `vertical-text`
- `alias`
- `copy`
- `move`
- `no-drop`
- `not-allowed`
- `grab`
- `grabbing`
- `e-resize`
- `n-resize`
- `ne-resize`
- `nw-resize`
- `s-resize`
- `se-resize`
- `sw-resize`
- `w-resize`
- `ew-resize`
- `ns-resize`
- `nesw-resize`
- `nwse-resize`
- `col-resize`
- `row-resize`
- `all-scroll`
- `zoom-in`
- `zoom-out`

## Response

Returns `CmdResultWindowSetCursorIcon`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the icon was set |
| message | String | Status or error message  |
