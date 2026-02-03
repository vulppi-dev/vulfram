# CmdLightUpdate

Updates an existing light's properties.

## Arguments

| Field          | Type              | Description                                                                         |
| -------------- | ----------------- | ----------------------------------------------------------------------------------- |
| windowId       | u32               | ID of the window                                                                    |
| lightId        | u32               | ID of the light to update                                                           |
| label          | Option<String>    | (Optional) New semantic name                                                        |
| kind           | Option<LightKind> | (Optional) New light type ("point", "directional", "spot", "ambient", "hemisphere") |
| position       | Option<Vec4>      | (Optional) New position                                                             |
| direction      | Option<Vec4>      | (Optional) New direction                                                            |
| color          | Option<Vec4>      | (Optional) New color                                                                |
| groundColor    | Option<Vec4>      | (Optional) New ground color                                                         |
| intensity      | Option<f32>       | (Optional) New intensity                                                            |
| range          | Option<f32>       | (Optional) New range                                                                |
| spotInnerOuter | Option<Vec2>      | (Optional) New spot angles                                                          |
| layerMask      | Option<u32>       | (Optional) New visibility mask                                                      |
| castShadow     | Option<bool>      | (Optional) New shadow casting state                                                 |

## Response

Returns `CmdResultLightUpdate`:

| Field   | Type   | Description                   |
| ------- | ------ | ----------------------------- |
| success | bool   | Whether the light was updated |
| message | String | Status or error message       |
