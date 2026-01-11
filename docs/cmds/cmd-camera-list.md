# CmdCameraList

Lists all cameras currently registered in the engine.

## Arguments

This command takes no arguments.

## Response

Returns `CmdResultCameraList`:

| Field     | Type               | Description                    |
| --------- | ------------------ | ------------------------------ |
| success   | bool               | Whether the list was retrieved |
| message   | String             | Status or error message        |
| resources | Vec<ResourceEntry> | List of camera metadata        |

### ResourceEntry

- **id**: u32 (Camera ID)
- **label**: Option<String> (Semantic name)
