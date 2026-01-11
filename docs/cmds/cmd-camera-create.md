# CmdCameraCreate

Creates a new camera resource.

## Arguments

| Field        | Type                 | Description                                    |
| ------------ | -------------------- | ---------------------------------------------- |
| cameraId     | u32                  | Unique ID for the camera                       |
| label        | Option<String>       | Optional semantic name for debugging/listing   |
| transform    | Mat4                 | Matrix for camera view transformation          |
| kind         | CameraKind           | Type of camera (Orthographic, Perspective)     |
| flags        | u32                  | Bitmask for camera options                     |
| nearFar      | Vec2                 | Near and far clipping planes [near, far]       |
| layerMask    | u32                  | Bitmask for visibility filtering               |
| order        | i32                  | Rendering order/priority                       |
| viewPosition | Option<ViewPosition> | Optional relative screen positioning           |
| orthoScale   | f32                  | Scale for orthographic cameras (default: 10.0) |

## Response

Returns `CmdResultCameraCreate`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the camera was created |
| message | String | Status or error message        |
