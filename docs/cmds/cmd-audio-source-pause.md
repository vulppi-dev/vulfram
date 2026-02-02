# CmdAudioSourcePause

Pauses playback for a source.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| sourceId | u32  | Source ID   |

## Response

Returns `CmdResultAudioSourcePause`:

| Field   | Type   | Description            |
| ------- | ------ | ---------------------- |
| success | bool   | Whether pause succeeded|
| message | String | Status or error message|
