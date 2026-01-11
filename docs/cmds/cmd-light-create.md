# CmdLightCreate

Creates a new light source in the scene.

## Arguments

| Field          | Type              | Description                                  |
| -------------- | ----------------- | -------------------------------------------- |
| windowId       | u32               | ID of the window                             |
| lightId        | u32               | Unique ID for the light                      |
| label          | Option<String>    | Optional semantic name for debugging/listing |
| kind           | Option<LightKind> | Type of light (Point, Directional, Spot)     |
| position       | Option<Vec4>      | Light position                               |
| direction      | Option<Vec4>      | Light direction (for directional/spot)       |
| color          | Option<Vec4>      | Light color (RGBA)                           |
| groundColor    | Option<Vec4>      | Ambient ground color                         |
| intensity      | Option<f32>       | Light brightness                             |
| range          | Option<f32>       | Effective distance                           |
| spotInnerOuter | Option<Vec2>      | Inner and outer spot angles                  |
| layerMask      | u32               | Visibility bitmask                           |
| castShadow     | bool              | Whether this light casts shadows             |

## Response

Returns `CmdResultLightCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was created |
| message | String | Status or error message       |
