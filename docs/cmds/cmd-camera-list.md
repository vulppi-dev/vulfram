# CmdCameraList

Lists all cameras registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultCameraList`:

| Field   | Type               | Description                    |
| ------- | ------------------ | ------------------------------ |
| success | bool               | Whether the list was retrieved |
| message | String             | Status or error message        |
| cameras | Vec<ResourceEntry> | List of camera metadata        |

### ResourceEntry

- **id**: u32 (Camera ID)
- **label**: Option<String> (Semantic name)
