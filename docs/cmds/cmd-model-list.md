# CmdModelList

Lists all models registered in a specific window.

## Arguments

| Field    | Type | Description      |
| -------- | ---- | ---------------- |
| windowId | u32  | ID of the window |

## Response

Returns `CmdResultModelList`:

| Field   | Type               | Description                    |
| ------- | ------------------ | ------------------------------ |
| success | bool               | Whether the list was retrieved |
| message | String             | Status or error message        |
| models  | Vec<ResourceEntry> | List of model metadata         |

### ResourceEntry

- **id**: u32 (Model ID)
- **label**: Option<String> (Semantic name)
