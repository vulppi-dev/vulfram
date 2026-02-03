# CmdAudioResourceDispose

Disposes an audio resource. Any playing layers using it are stopped.

## Arguments

| Field      | Type | Description                  |
| ---------- | ---- | ---------------------------- |
| resourceId | u32  | Audio resource ID to dispose |

## Response

Returns `CmdResultAudioResourceDispose`:

| Field   | Type   | Description              |
| ------- | ------ | ------------------------ |
| success | bool   | Whether dispose succeeded|
| message | String | Status or error message  |
