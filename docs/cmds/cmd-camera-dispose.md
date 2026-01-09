# CmdCameraDispose

Disposes of a camera resource.

## Arguments

| Field    | Type | Description                |
| -------- | ---- | -------------------------- |
| cameraId | u32  | ID of the camera to remove |

## Response

Returns `CmdResultCameraDispose`:

| Field   | Type   | Description                    |
| ------- | ------ | ------------------------------ |
| success | bool   | Whether the camera was removed |
| message | String | Status or error message        |
