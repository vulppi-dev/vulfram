# CmdGeometryList

Lists all geometries registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultGeometryList`:

| Field     | Type               | Description                    |
| --------- | ------------------ | ------------------------------ |
| success   | bool               | Whether the list was retrieved |
| message   | String             | Status or error message        |
| resources | Vec<ResourceEntry> | List of geometry metadata      |

### ResourceEntry

- **id**: u32 (Geometry ID)
- **label**: Option<String> (Semantic name)
