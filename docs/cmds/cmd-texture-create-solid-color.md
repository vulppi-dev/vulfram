# CmdTextureCreateSolidColor

Creates a 1x1 texture with a solid color.

## Arguments

| Field        | Type                        | Description                                         |
| ------------ | --------------------------- | --------------------------------------------------- |
| windowId     | u32                         | ID of the window                                    |
| textureId    | u32                         | Unique ID for the texture                           |
| label        | Option<String>              | (Optional) Semantic name                            |
| color        | Vec4                        | Color in RGBA                                       |
| srgb         | Option<bool>                | (Optional) Use sRGB (default: true)                 |
| mode         | TextureCreateMode           | (Optional) Standalone (0) or ForwardAtlas (1) (default: 0) |
| atlasOptions | Option<ForwardAtlasOptions> | (Optional) Options for atlas allocation             |

### ForwardAtlasOptions

- **tilePx**: u32 (tile size in pixels, default: 256)
- **layers**: u32 (atlas layers, default: 1)

## Response

Returns `CmdResultTextureCreateSolidColor`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was created |
| message | String | Status or error message         |
