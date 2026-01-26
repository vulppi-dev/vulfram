# CmdMaterialCreate

Creates a new material resource ("standard" or "pbr").

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
| kind       | MaterialKind            | Type of material ("standard", "pbr")     |
| options    | Option<MaterialOptions> | (Optional) StandardOptions or PbrOptions |

### SurfaceType (Enum)

- `opaque`
- `masked`
- `transparent`

### MaterialSampler (Enum)

- `point-clamp`
- `linear-clamp`
- `point-repeat`
- `linear-repeat`

### StandardOptions

- **baseColor**: Vec4
- **surfaceType**: SurfaceType
- **specColor**: Option<Vec4>
- **specPower**: Option<f32>
- **baseTexId**: Option<u32>
- **baseSampler**: Option<MaterialSampler>
- **specTexId**: Option<u32>
- **specSampler**: Option<MaterialSampler>
- **normalTexId**: Option<u32>
- **normalSampler**: Option<MaterialSampler>
- **toonRampTexId**: Option<u32>
- **toonRampSampler**: Option<MaterialSampler>
- **flags**: u32
- **toonParams**: Option<Vec4>

### PbrOptions

- **baseColor**: Vec4
- **surfaceType**: SurfaceType
- **emissiveColor**: Vec4
- **metallic**: f32
- **roughness**: f32
- **ao**: f32
- **normalScale**: f32
- **baseTexId**: Option<u32>
- **baseSampler**: Option<MaterialSampler>
- **normalTexId**: Option<u32>
- **normalSampler**: Option<MaterialSampler>
- **metallicRoughnessTexId**: Option<u32>
- **metallicRoughnessSampler**: Option<MaterialSampler>
- **emissiveTexId**: Option<u32>
- **emissiveSampler**: Option<MaterialSampler>
- **aoTexId**: Option<u32>
- **aoSampler**: Option<MaterialSampler>
- **flags**: u32

## Response

Returns `CmdResultMaterialCreate`:

| Field   | Type   | Description                      |
| ------- | ------ | -------------------------------- |
| success | bool   | Whether the material was created |
| message | String | Status or error message          |
