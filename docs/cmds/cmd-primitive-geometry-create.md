# CmdPrimitiveGeometryCreate

Generates a standard primitive shape (Cube, Sphere, etc.) as a geometry resource.

## Arguments

| Field      | Type                     | Description                                               |
| ---------- | ------------------------ | --------------------------------------------------------- |
| windowId   | u32                      | ID of the window                                          |
| geometryId | u32                      | ID for the generated geometry                             |
| label      | Option<String>           | (Optional) Semantic name                                  |
| shape      | PrimitiveShape           | "cube", "plane", "sphere", "cylinder", "torus", "pyramid" |
| options    | Option<PrimitiveOptions> | (Optional) Parameters (size, radius, subdivisions, etc.)  |

## Response

Returns `CmdResultPrimitiveGeometryCreate`:

| Field   | Type   | Description                         |
| ------- | ------ | ----------------------------------- |
| success | bool   | Whether the primitive was generated |
| message | String | Status or error message             |
