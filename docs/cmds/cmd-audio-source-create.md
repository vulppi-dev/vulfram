# CmdAudioSourceCreate

Creates an audio source bound to a model.

## Arguments

| Field       | Type               | Description                     |
| ----------- | ------------------ | ------------------------------- |
| windowId    | u32                | Window owning the model         |
| sourceId    | u32                | Logical source ID               |
| resourceId  | u32                | Audio resource ID               |
| modelId     | u32                | Model to bind as emitter         |
| position    | Vec3               | Initial position (fallback)     |
| velocity    | Vec3               | Initial velocity                |
| orientation | Quat               | Initial orientation             |
| gain        | f32                | Base gain (volume)              |
| pitch       | f32                | Playback rate factor            |
| spatial     | AudioSpatialParams | Spatial parameters              |

## Response

Returns `CmdResultAudioSourceCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the source was created|
| message | String | Status or error message       |
