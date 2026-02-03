# CmdAudioListenerUpdate

Updates the listener transform explicitly.

## Arguments

| Field    | Type | Description                    |
| -------- | ---- | ------------------------------ |
| position | Vec3 | Listener position in world     |
| velocity | Vec3 | Listener velocity (optional)  |
| forward  | Vec3 | Forward direction (unit-ish)  |
| up       | Vec3 | Up direction (unit-ish)       |

## Response

Returns `CmdResultAudioListenerUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the update succeeded  |
| message | String | Status or error message       |
