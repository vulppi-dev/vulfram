# CmdMaterialList

Lists all materials registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultMaterialList`:

| Field     | Type               | Description                    |
| --------- | ------------------ | ------------------------------ |
| success   | bool               | Whether the list was retrieved |
| message   | String             | Status or error message        |
| materials | Vec<ResourceEntry> | List of material metadata      |

### ResourceEntry

- **id**: u32 (Material ID)
- **label**: Option<String> (Semantic name)
