# CmdAudioSourcePause

Pauses playback for a source. The layer is preserved for resuming.

## Arguments

| Field      | Type        | Description                               |
| ---------- | ----------- | ----------------------------------------- |
| sourceId   | u32         | Source ID                                 |
| timelineId | Option<u32> | Optional timeline layer (defaults to all) |

## Response

Returns `CmdResultAudioSourcePause`:

| Field   | Type   | Description             |
| ------- | ------ | ----------------------- |
| success | bool   | Whether pause succeeded |
| message | String | Status or error message |
