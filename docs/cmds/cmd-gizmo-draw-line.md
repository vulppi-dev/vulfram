# CmdGizmoDrawLine

Draws a 3D line gizmo in the scene. Gizmos are cleared every frame.

## Arguments

| Field | Type | Description                |
| ----- | ---- | -------------------------- |
| start | Vec3 | Starting point of the line |
| end   | Vec3 | Ending point of the line   |
| color | Vec4 | Color of the line (RGBA)   |

## Response

Returns `CmdResultGizmoDraw`:

| Field  | Type | Description                 |
| ------ | ---- | --------------------------- |
| status | u32  | Status code (0 for success) |
