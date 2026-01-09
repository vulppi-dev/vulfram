# CmdPrimitiveGeometryCreate

Generates a standard primitive shape (Cube, Sphere, etc.) as a geometry resource.

## Arguments

| Field      | Type                     | Description                                                  |
| ---------- | ------------------------ | ------------------------------------------------------------ |
| windowId   | u32                      | ID of the window                                             |
| geometryId | u32                      | ID for the generated geometry                                |
| shape      | PrimitiveShape           | Cube, Plane, Sphere, Cylinder, Torus, Pyramid                |
| options    | Option<PrimitiveOptions> | Shape-specific parameters (size, radius, subdivisions, etc.) |

## Response

Returns `CmdResultPrimitiveGeometryCreate`:

| Field   | Type   | Description                         |
| ------- | ------ | ----------------------------------- |
| success | bool   | Whether the primitive was generated |
| message | String | Status or error message             |
