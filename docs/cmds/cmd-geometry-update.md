# CmdGeometryUpdate

Updates a geometry resource. Replaces all previous data with new buffers.

## Arguments

| Field      | Type                        | Description                  |
| ---------- | --------------------------- | ---------------------------- |
| windowId   | u32                         | ID of the window             |
| geometryId | u32                         | ID of the geometry to update |
| entries    | Vec<GeometryPrimitiveEntry> | New set of primitive buffers |

## Response

Returns `CmdResultGeometryUpdate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the geometry was updated |
| message | String | Status or error message          |
