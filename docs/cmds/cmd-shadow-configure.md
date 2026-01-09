# CmdShadowConfigure

Configures global shadow mapping settings for a window.

## Arguments

| Field | Type | Description |
|-------|------|-------------|
| windowId | u32 | ID of the window |
| config | ShadowConfig | Shadow configuration parameters |

### ShadowConfig
- **enabled**: bool
- **atlasSize**: u32
- **renderType**: Standard, Pcf, etc.
- **distance**: f32
- **bias**: f32

## Response

Returns `CmdResultShadowConfigure`:

| Field | Type | Description |
|-------|------|-------------|
| success | bool | Whether the configuration was updated |
| message | String | Status or error message |
