# CmdTextureCreateSolidColor

Creates a 1x1 texture with a solid color.

## Arguments

| Field     | Type           | Description                                  |
| --------- | -------------- | -------------------------------------------- |
| windowId  | u32            | ID of the window                             |
| textureId | u32            | Unique ID for the texture                    |
| label     | Option<String> | Optional semantic name for debugging/listing |
| color     | Vec4           | Color in RGBA                                |

## Response

Returns `CmdResultTextureCreateSolidColor`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was created |
| message | String | Status or error message         |
