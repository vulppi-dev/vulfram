# CmdTextureDispose

Removes a texture resource.

## Arguments

| Field     | Type | Description                 |
| --------- | ---- | --------------------------- |
| windowId  | u32  | ID of the window            |
| textureId | u32  | ID of the texture to remove |

## Response

Returns `CmdResultTextureDispose`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was removed |
| message | String | Status or error message         |
