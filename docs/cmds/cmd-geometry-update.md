# CmdGeometryUpdate

Updates a geometry resource.

Behavior:

- If `entries` is `None`, only the label is updated (if provided).
- If `entries` is present, geometry data is replaced with the new buffers.

Validation rules for `entries`:

- `Position` is required.
- Maximum of 2 `UV` streams.
- Duplicates (except `UV`) are rejected.
- All referenced `bufferId`s must exist in the upload table.

Buffers are only removed from the upload table after a successful update.

## Arguments

| Field      | Type                                | Description                             |
| ---------- | ----------------------------------- | --------------------------------------- |
| windowId   | u32                                 | ID of the window                        |
| geometryId | u32                                 | ID of the geometry to update            |
| label      | Option<String>                      | (Optional) New semantic name            |
| entries    | Option<Vec<GeometryPrimitiveEntry>> | (Optional) New set of primitive buffers |

## Response

Returns `CmdResultGeometryUpdate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the geometry was updated |
| message | String | Status or error message          |
