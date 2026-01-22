# CmdWindowSetIcon

Sets the icon of a window using an uploaded image buffer.

## Platform Notes

- **WASM:** Not supported (returns `success=false` with a message).

## Notes

- `bufferId` must refer to an upload with `UploadType::ImageData`.
- Supported formats: PNG, JPEG, WebP, AVIF.

## Arguments

| Field    | Type | Description                                     |
| -------- | ---- | ----------------------------------------------- |
| windowId | u32  | ID of the window                                |
| bufferId | u64  | ID of the uploaded buffer containing image data |

## Response

Returns `CmdResultWindowSetIcon`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether the icon was set |
| message | String | Status or error message  |
