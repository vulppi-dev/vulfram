# CmdLightList

Lists all lights registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultLightList`:

| Field     | Type               | Description                    |
| --------- | ------------------ | ------------------------------ |
| success   | bool               | Whether the list was retrieved |
| message   | String             | Status or error message        |
| resources | Vec<ResourceEntry> | List of light metadata         |

### ResourceEntry

- **id**: u32 (Light ID)
- **label**: Option<String> (Semantic name)
