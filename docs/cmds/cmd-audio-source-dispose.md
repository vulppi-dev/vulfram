# CmdAudioSourceDispose

Disposes a single source and its binding.

## Arguments

| Field    | Type | Description |
| -------- | ---- | ----------- |
| sourceId | u32  | Source ID   |

## Response

Returns `CmdResultAudioSourceDispose`:

| Field   | Type   | Description               |
| ------- | ------ | ------------------------- |
| success | bool   | Whether dispose succeeded |
| message | String | Status or error message   |
