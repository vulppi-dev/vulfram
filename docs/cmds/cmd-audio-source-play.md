# CmdAudioSourcePlay

Starts playback for a source.

## Arguments

| Field     | Type         | Description                                         |
| --------- | ------------ | --------------------------------------------------- |
| sourceId  | u32          | Source ID                                           |
| intensity | f32          | Extra volume multiplier (0..1)                      |
| delayMs   | Option<u32>  | Optional delay in milliseconds                      |
| mode      | AudioPlayMode| once | loop | reverse | loop-reverse | ping-pong |

## Response

Returns `CmdResultAudioSourcePlay`:

| Field   | Type   | Description                |
| ------- | ------ | -------------------------- |
| success | bool   | Whether playback started   |
| message | String | Status or error message     |
