# CmdCameraUpdate

Updates an existing camera's properties. All fields are optional and only provided ones will be updated.

## Arguments

| Field        | Type                 | Description                       |
| ------------ | -------------------- | --------------------------------- |
| cameraId     | u32                  | ID of the camera to update        |
| label        | Option<String>       | (Optional) New semantic name      |
| transform    | Option<Mat4>         | (Optional) New view matrix        |
| kind         | Option<CameraKind>   | (Optional) New camera type ("orthographic", "perspective") |
| flags        | Option<u32>          | (Optional) New camera flags       |
| nearFar      | Option<Vec2>         | (Optional) New clipping planes    |
| layerMask    | Option<u32>          | (Optional) New visibility mask    |
| order        | Option<i32>          | (Optional) New rendering order    |
| viewPosition | Option<ViewPosition> | (Optional) New screen positioning |
| orthoScale   | Option<f32>          | (Optional) New ortho scale        |

## Response

Returns `CmdResultCameraUpdate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the camera was updated |
| message | String | Status or error message        |
