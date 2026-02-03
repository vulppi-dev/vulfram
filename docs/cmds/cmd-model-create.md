# CmdModelCreate

Creates a new model instance.

Geometry and material references are **lazy**:

- `geometryId` does not need to exist at creation time.
- `materialId` does not need to exist at creation time.

If geometry/material are missing, the model renders with fallbacks (or is skipped)
until those resources appear later with the same IDs.

## Arguments

| Field         | Type           | Description                                                    |
| ------------- | -------------- | -------------------------------------------------------------- |
| windowId      | u32            | ID of the window where this model belongs                      |
| modelId       | u32            | Unique ID for the model                                        |
| label         | Option<String> | (Optional) Semantic name for debugging/listing                 |
| geometryId    | u32            | ID of the geometry resource to use (may not exist yet)          |
| materialId    | Option<u32>    | (Optional) ID of the material resource (may not exist yet)      |
| transform     | Mat4           | Model transformation matrix (world position/rotation/scale)    |
| layerMask     | u32            | (Optional) Visibility bitmask (default: 0xFFFFFFFF)            |
| castShadow    | bool           | (Optional) Whether this model casts shadows (default: true)    |
| receiveShadow | bool           | (Optional) Whether this model receives shadows (default: true) |
| castOutline   | bool           | (Optional) Whether this model writes to the outline mask (default: false) |
| outlineColor  | Vec4           | (Optional) Outline color written into the outline mask (default: 0,0,0,0) |

## Response

Returns `CmdResultModelCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the model was created |
| message | String | Status or error message       |
