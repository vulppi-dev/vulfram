# CmdCameraUpdate

Updates an existing camera's properties. All fields are optional and only provided ones will be updated.

## Arguments

| Field        | Type                 | Description                     |
| ------------ | -------------------- | ------------------------------- |
| cameraId     | u32                  | ID of the camera to update      |
| transform    | Option<Mat4>         | New view transformation matrix  |
| kind         | Option<CameraKind>   | New camera type                 |
| flags        | Option<u32>          | New camera flags                |
| nearFar      | Option<Vec2>         | New clipping planes [near, far] |
| layerMask    | Option<u32>          | New visibility mask             |
| order        | Option<i32>          | New rendering order             |
| viewPosition | Option<ViewPosition> | New relative positioning        |
| orthoScale   | Option<f32>          | New orthographic scale          |

## Response

Returns `CmdResultCameraUpdate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the camera was updated |
| message | String | Status or error message        |
