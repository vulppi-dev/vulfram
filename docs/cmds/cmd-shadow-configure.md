# CmdShadowConfigure

Configures global shadow mapping settings for a window.

## Arguments

| Field    | Type         | Description                     |
| -------- | ------------ | ------------------------------- |
| windowId | u32          | ID of the window                |
| config   | ShadowConfig | Shadow configuration parameters |

### ShadowConfig

| Field           | Type | Description                                                   |
| --------------- | ---- | ------------------------------------------------------------- |
| tileResolution  | u32  | Size of each shadow tile (default: 1024)                      |
| atlasTilesW     | u32  | Number of tiles horizontally in the atlas (default: 8)        |
| atlasTilesH     | u32  | Number of tiles vertically in the atlas (default: 8)          |
| atlasLayers     | u32  | Number of atlas layers/textures (default: 1)                  |
| virtualGridSize | u32  | Grid size for shadow clustering/assignment (default: 1)       |
| smoothing       | u32  | Percentage of tile resolution for PCF kernels (default: 1)    |
| normalBias      | f32  | World-space normal offset for shadow sampling (default: 0.01) |

All fields are optional when sending from host (will use defaults).

## Response

Returns `CmdResultShadowConfigure`:

| Field   | Type   | Description                           |
| ------- | ------ | ------------------------------------- |
| success | bool   | Whether the configuration was updated |
| message | String | Status or error message               |
