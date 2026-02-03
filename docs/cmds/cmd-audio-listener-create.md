# CmdAudioListenerCreate

Creates (binds) the listener to a model so its transform drives the listener each tick.

## Arguments

| Field    | Type | Description               |
| -------- | ---- | ------------------------- |
| windowId | u32  | Window owning the model   |
| modelId  | u32  | Model to bind as listener |

## Response

Returns `CmdResultAudioListenerCreate`:

| Field   | Type   | Description                |
| ------- | ------ | -------------------------- |
| success | bool   | Whether the bind succeeded |
| message | String | Status or error message    |
