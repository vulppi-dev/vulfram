# CmdAudioSourceStop

Stops playback for a source. This destroys the layer(s).

If `timelineId` is omitted, all layers are stopped.

## Arguments

| Field      | Type        | Description                             |
| ---------- | ----------- | --------------------------------------- |
| sourceId   | u32         | Source ID                               |
| timelineId | Option<u32> | Optional timeline layer (defaults to all)|

## Response

Returns `CmdResultAudioSourceStop`:

| Field   | Type   | Description           |
| ------- | ------ | --------------------- |
| success | bool   | Whether stop succeeded|
| message | String | Status or error message|
