# CmdMaterialDispose

Removes a material resource.

## Arguments

| Field      | Type | Description                  |
| ---------- | ---- | ---------------------------- |
| windowId   | u32  | ID of the window             |
| materialId | u32  | ID of the material to remove |

## Response

Returns `CmdResultMaterialDispose`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the material was removed |
| message | String | Status or error message          |
