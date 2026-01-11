# CmdTextureList

Lists all textures registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultTextureList`:

| Field     | Type               | Description                    |
| --------- | ------------------ | ------------------------------ |
| success   | bool               | Whether the list was retrieved |
| message   | String             | Status or error message        |
| resources | Vec<ResourceEntry> | List of texture metadata       |

### ResourceEntry

- **id**: u32 (Texture ID)
- **label**: Option<String> (Semantic name)
