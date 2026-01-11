# CmdModelUpdate

Updates an existing model's properties.

## Arguments

| Field         | Type           | Description                     |
| ------------- | -------------- | ------------------------------- |
| windowId      | u32            | ID of the window                |
| modelId       | u32            | ID of the model to update       |
| label         | Option<String> | New semantic name for the model |
| geometryId    | Option<u32>    | New geometry ID                 |
| materialId    | Option<u32>    | New material ID                 |
| transform     | Option<Mat4>   | New transformation matrix       |
| layerMask     | Option<u32>    | New visibility mask             |
| castShadow    | Option<bool>   | Whether to cast shadows         |
| receiveShadow | Option<bool>   | Whether to receive shadows      |

## Response

Returns `CmdResultModelUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was updated |
| message | String | Status or error message       |
