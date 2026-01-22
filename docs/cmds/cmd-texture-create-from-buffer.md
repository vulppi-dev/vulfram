# CmdTextureCreateFromBuffer

Creates a texture from an uploaded image buffer.

Notes:

- `bufferId` must refer to an upload with `UploadType::ImageData`.
- Supported formats: PNG, JPEG, WebP, AVIF.
- If `mode` is `ForwardAtlas`, creation can fail if an atlas already exists
  with a different configuration.

## Arguments

| Field        | Type                        | Description                                         |
| ------------ | --------------------------- | --------------------------------------------------- |
| windowId     | u32                         | ID of the window                                    |
| textureId    | u32                         | Unique ID for the texture                           |
| label        | Option<String>              | (Optional) Semantic name                            |
| bufferId     | u64                         | ID of the uploaded buffer containing image data     |
| srgb         | Option<bool>                | (Optional) Use sRGB (default: true)                 |
| mode         | TextureCreateMode           | (Optional) Standalone (0) or ForwardAtlas (1) (default: 0) |
| atlasOptions | Option<ForwardAtlasOptions> | (Optional) Options for atlas allocation             |

### ForwardAtlasOptions

- **tilePx**: u32 (tile size in pixels, default: 256)
- **layers**: u32 (atlas layers, default: 1)

## Response

Returns `CmdResultTextureCreateFromBuffer`:

| Field   | Type   | Description                     |
| ------- | ------ | ------------------------------- |
| success | bool   | Whether the texture was created |
| message | String | Status or error message         |
