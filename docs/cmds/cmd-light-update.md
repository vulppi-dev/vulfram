# CmdLightUpdate

Updates an existing light's properties.

## Arguments

| Field          | Type              | Description               |
| -------------- | ----------------- | ------------------------- |
| windowId       | u32               | ID of the window          |
| lightId        | u32               | ID of the light to update |
| kind           | Option<LightKind> | New light type            |
| position       | Option<Vec4>      | New position              |
| direction      | Option<Vec4>      | New direction             |
| color          | Option<Vec4>      | New color                 |
| groundColor    | Option<Vec4>      | New ground color          |
| intensity      | Option<f32>       | New intensity             |
| range          | Option<f32>       | New range                 |
| spotInnerOuter | Option<Vec2>      | New spot angles           |
| layerMask      | Option<u32>       | New visibility mask       |
| castShadow     | Option<bool>      | New shadow casting state  |

## Response

Returns `CmdResultLightUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was updated |
| message | String | Status or error message       |
