# CmdAudioSourceStop

Stops playback for a source.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| sourceId | u32  | Source ID   |

## Response

Returns `CmdResultAudioSourceStop`:

| Field   | Type   | Description           |
| ------- | ------ | --------------------- |
| success | bool   | Whether stop succeeded|
| message | String | Status or error message|
