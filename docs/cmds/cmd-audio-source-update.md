# CmdAudioSourceUpdate

Updates source spatial parameters and gain/pitch.

## Arguments

| Field       | Type               | Description          |
| ----------- | ------------------ | -------------------- |
| sourceId    | u32                | Source ID            |
| position    | Vec3               | Position             |
| velocity    | Vec3               | Velocity             |
| orientation | Quat               | Orientation          |
| gain        | f32                | Base gain            |
| pitch       | f32                | Playback rate factor |
| spatial     | AudioSpatialParams | Spatial parameters   |

## Response

Returns `CmdResultAudioSourceUpdate`:

| Field   | Type   | Description                  |
| ------- | ------ | ---------------------------- |
| success | bool   | Whether the update succeeded |
| message | String | Status or error message      |
