# CmdModelUpdate

Updates an existing model's properties.

Geometry and material references are **lazy**:

- `geometryId` may point to a resource that does not exist yet.
- `materialId` may point to a resource that does not exist yet.

If geometry/material are missing, the model renders with fallbacks (or is skipped)
until those resources appear later with the same IDs.

## Arguments

| Field         | Type           | Description                           |
| ------------- | -------------- | ------------------------------------- |
| windowId      | u32            | ID of the window                      |
| modelId       | u32            | ID of the model to update             |
| label         | Option<String> | (Optional) New semantic name          |
| geometryId    | Option<u32>    | (Optional) New geometry ID (may not exist yet) |
| materialId    | Option<u32>    | (Optional) New material ID (may not exist yet) |
| transform     | Option<Mat4>   | (Optional) New transform matrix       |
| layerMask     | Option<u32>    | (Optional) New visibility mask        |
| castShadow    | Option<bool>   | (Optional) New shadow casting state   |
| receiveShadow | Option<bool>   | (Optional) New shadow receiving state |

## Response

Returns `CmdResultModelUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was updated |
| message | String | Status or error message       |
