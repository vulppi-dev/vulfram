# CmdGeometryCreate

Creates a geometry resource from multiple uploaded vertex/index buffers.

Validation rules:

- `Position` is required.
- Maximum of 2 `UV` streams.
- Duplicates (except `UV`) are rejected.
- All referenced `bufferId`s must exist in the upload table.

Buffers are only removed from the upload table after a successful create.

## Arguments

| Field      | Type                        | Description                               |
| ---------- | --------------------------- | ----------------------------------------- |
| windowId   | u32                         | ID of the window                          |
| geometryId | u32                         | Unique ID for the geometry                |
| label      | Option<String>              | (Optional) Semantic name                  |
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
