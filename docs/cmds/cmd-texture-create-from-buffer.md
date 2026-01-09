# CmdTextureCreateFromBuffer

Creates a texture from an uploaded image buffer.

## Arguments

| Field        | Type                        | Description                                     |
| ------------ | --------------------------- | ----------------------------------------------- |
| windowId     | u32                         | ID of the window                                |
| textureId    | u32                         | Unique ID for the texture                       |
| bufferId     | u64                         | ID of the uploaded buffer containing image data |
| srgb         | Option<bool>                | Whether to use sRGB format (default: true)      |
| mode         | TextureCreateMode           | Standalone (0) or ForwardAtlas (1)              |
| atlasOptions | Option<ForwardAtlasOptions> | Options for atlas allocation                    |

## Response

Returns `CmdResultTextureCreateFromBuffer`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was created |
| message | String | Status or error message         |
