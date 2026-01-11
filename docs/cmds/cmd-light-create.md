# CmdLightCreate

Creates a new light source in the scene.

## Arguments

| Field          | Type              | Description                                |
| -------------- | ----------------- | ------------------------------------------ |
| windowId       | u32               | ID of the window                           |
| lightId        | u32               | Unique ID for the light                    |
| label          | Option<String>    | (Optional) Semantic name                   |
| kind           | Option<LightKind> | (Optional) Type (Point, Directional, Spot) |
| position       | Option<Vec4>      | (Optional) Light position                  |
| direction      | Option<Vec4>      | (Optional) Light direction                 |
| color          | Option<Vec4>      | (Optional) Light color (RGBA)              |
| groundColor    | Option<Vec4>      | (Optional) Ambient ground color            |
| intensity      | Option<f32>       | (Optional) Light brightness                |
| range          | Option<f32>       | (Optional) Effective distance              |
| spotInnerOuter | Option<Vec2>      | (Optional) Inner and outer spot angles     |
| layerMask      | u32               | (Optional) Mask (default: 0xFFFFFFFF)      |
| castShadow     | bool              | (Optional) Cast shadows (default: true)    |

## Response

Returns `CmdResultLightCreate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was created |
| message | String | Status or error message       |
