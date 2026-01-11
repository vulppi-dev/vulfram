# CmdMaterialUpdate

Updates an existing material's properties.

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
