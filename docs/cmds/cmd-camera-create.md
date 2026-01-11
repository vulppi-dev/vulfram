# CmdCameraCreate

Creates a new camera resource.

## Arguments

| Field        | Type                 | Description                                        |
| ------------ | -------------------- | -------------------------------------------------- |
| cameraId     | u32                  | Unique ID for the camera                           |
| label        | Option<String>       | (Optional) Semantic name                           |
| transform    | Mat4                 | Matrix for camera view transformation              |
| kind         | CameraKind           | Type of camera (Orthographic, Perspective)         |
| flags        | u32                  | (Optional) Bitmask for camera options (default: 0) |
| nearFar      | Vec2                 | Near and far clipping planes [near, far]           |
| layerMask    | u32                  | (Optional) Visibility mask (default: 0xFFFFFFFF)   |
| order        | i32                  | (Optional) Rendering order (default: 0)            |
| viewPosition | Option<ViewPosition> | (Optional) Relative screen positioning             |
| orthoScale   | f32                  | (Optional) Ortho scale (default: 10.0)             |

## Response

Returns `CmdResultCameraCreate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the camera was created |
| message | String | Status or error message        |
