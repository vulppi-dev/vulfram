# CmdMaterialCreate

Creates a new material resource (Standard or PBR).

Texture references are **lazy**:

- Any texture ID inside `options` may refer to a texture that does not exist yet.
- Missing textures render with fallbacks until the texture appears later with
  the same ID.

## Arguments

| Field      | Type                    | Description                              |
| ---------- | ----------------------- | ---------------------------------------- |
| windowId   | u32                     | ID of the window                         |
| materialId | u32                     | Unique ID for the material               |
| label      | Option<String>          | (Optional) Semantic name                 |
| kind       | MaterialKind            | Type of material (Standard, Pbr)         |
| options    | Option<MaterialOptions> | (Optional) StandardOptions or PbrOptions |

### StandardOptions

- **baseColor**: Vec4
- **surfaceType**: Opaque, Transparent, Cutout
- **specColor**: Option<Vec4>
- **specPower**: Option<f32>
- **baseTexId**: Option<u32> (may not exist yet)
- **normalTexId**: Option<u32> (may not exist yet)
- ... (and more texture/sampler options)

### PbrOptions

- **baseColor**: Vec4
- **emissiveColor**: Vec4
- **metallic**: f32
- **roughness**: f32
- **ao**: f32
- **normalScale**: f32
- **baseTexId**: Option<u32> (may not exist yet)
- **normalTexId**: Option<u32> (may not exist yet)
- ... (and more PBR specific textures)

## Response

Returns `CmdResultMaterialCreate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the material was created |
| message | String | Status or error message          |
