# CmdGeometryCreate

Creates a geometry resource from multiple uploaded vertex/index buffers.

## Arguments

| Field      | Type                        | Description                               |
| ---------- | --------------------------- | ----------------------------------------- |
| windowId   | u32                         | ID of the window                          |
| geometryId | u32                         | Unique ID for the geometry                |
| entries    | Vec<GeometryPrimitiveEntry> | List of buffers and their primitive types |

### GeometryPrimitiveEntry

- **primitiveType**: Position, Normal, UV, Color, Tangent, JointIndices, JointWeights, Index, etc.
- **bufferId**: u64 (ID of the uploaded buffer)

## Response

Returns `CmdResultGeometryCreate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the geometry was created |
| message | String | Status or error message          |
