# CmdAudioSourcePlay

Starts playback for a source.

If `timelineId` is already playing, it is stopped and restarted.

## Arguments

| Field      | Type          | Description                             |
| ---------- | ------------- | --------------------------------------- |
| sourceId   | u32           | Source ID                               |
| resourceId | u32           | Audio resource ID to play               |
| timelineId | Option<u32>   | Optional timeline layer (defaults to 0) |
| intensity  | f32           | Extra volume multiplier (0..1)          |
| delayMs    | Option<u32>   | Optional delay in milliseconds          |
| mode       | AudioPlayMode | once \| loop                            |

## Response

Returns `CmdResultAudioSourcePlay`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether playback started |
| message | String | Status or error message  |
