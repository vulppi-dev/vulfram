# CmdGeometryCreate

Creates a geometry resource from multiple uploaded vertex/index buffers.

Validation rules:

- `position` is required.
- Maximum of 2 `uv` streams.
- Duplicates (except `uv`) are rejected.
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

- **primitiveType**: "index", "position", "normal", "tangent", "color", "uv", "skin-joints", "skin-weights".
- **bufferId**: u64 (ID of the uploaded buffer)

## Response

Returns `CmdResultGeometryCreate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the geometry was created |
| message | String | Status or error message          |
