# CmdMaterialUpdate

Updates an existing material's properties.

Texture references are **lazy**:

- Any texture ID inside `options` may refer to a texture that does not exist yet.
- Missing textures render with fallbacks until the texture appears later with
  the same ID.

## Arguments

| Field      | Type                    | Description                     |
| ---------- | ----------------------- | ------------------------------- |
| windowId   | u32                     | ID of the window                |
| materialId | u32                     | ID of the material to update    |
| label      | Option<String>          | (Optional) New semantic name    |
| kind       | Option<MaterialKind>    | (Optional) New material type    |
| options    | Option<MaterialOptions> | (Optional) New material options |

## Response

Returns `CmdResultMaterialUpdate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the material was updated |
| message | String | Status or error message          |
