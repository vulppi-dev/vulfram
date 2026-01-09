# CmdModelCreate

Creates a new model instance (links geometry and material at a transform).

## Arguments

| Field         | Type        | Description                                                 |
| ------------- | ----------- | ----------------------------------------------------------- |
| windowId      | u32         | ID of the window where this model belongs                   |
| modelId       | u32         | Unique ID for the model                                     |
| geometryId    | u32         | ID of the geometry resource to use                          |
| materialId    | Option<u32> | ID of the material resource (optional)                      |
| transform     | Mat4        | Model transformation matrix (world position/rotation/scale) |
| layerMask     | u32         | Visibility bitmask                                          |
| castShadow    | bool        | Whether this model casts shadows                            |
| receiveShadow | bool        | Whether this model receives shadows                         |

## Response

Returns `CmdResultModelCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was created |
| message | String | Status or error message       |
