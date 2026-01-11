# CmdModelCreate

Creates a new model instance (links geometry and material at a transform).

## Arguments

| Field         | Type           | Description                                                    |
| ------------- | -------------- | -------------------------------------------------------------- |
| windowId      | u32            | ID of the window where this model belongs                      |
| modelId       | u32            | Unique ID for the model                                        |
| label         | Option<String> | (Optional) Semantic name for debugging/listing                 |
| geometryId    | u32            | ID of the geometry resource to use                             |
| materialId    | Option<u32>    | (Optional) ID of the material resource                         |
| transform     | Mat4           | Model transformation matrix (world position/rotation/scale)    |
| layerMask     | u32            | (Optional) Visibility bitmask (default: 0xFFFFFFFF)            |
| castShadow    | bool           | (Optional) Whether this model casts shadows (default: true)    |
| receiveShadow | bool           | (Optional) Whether this model receives shadows (default: true) |

## Response

Returns `CmdResultModelCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was created |
| message | String | Status or error message       |
