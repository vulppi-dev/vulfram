import type {
  TextureFormat,
  TextureUsage,
  VertexFormat,
  IndexFormat,
  AddressMode,
  FilterMode,
  MipmapFilterMode,
  CompareFunction,
  BorderColor,
} from '../enums';

// MARK: Logical IDs

export type ShaderId = number;
export type GeometryId = number;
export type MaterialId = number;
export type TextureId = number;
export type SamplerId = number;
export type ComponentId = number;

// MARK: Uniform Types

export type UniformType =
  | 'float'
  | 'int'
  | 'u-int'
  | 'bool'
  | 'vec2'
  | 'vec3'
  | 'vec4'
  | 'vec2i'
  | 'vec3i'
  | 'vec4i'
  | 'vec2u'
  | 'vec3u'
  | 'vec4u'
  | 'mat2x2'
  | 'mat2x3'
  | 'mat2x4'
  | 'mat3x2'
  | 'mat3x3'
  | 'mat3x4'
  | 'mat4x2'
  | 'mat4x3'
  | 'mat4x4'
  | 'atomic-int'
  | 'atomic-u-int';

export interface UniformField {
  name: string;
  uniformType: UniformType;
  arraySize?: number;
}

// MARK: Shader Bindings

export type TextureSampleType = 'float' | 'depth' | 'sint' | 'uint';

export type TextureViewDimension =
  | 'd1'
  | 'd2'
  | 'd2-array'
  | 'cube'
  | 'cube-array'
  | 'd3';

export interface TextureBinding {
  group: number;
  binding: number;
  sampleType: TextureSampleType;
  viewDimension: TextureViewDimension;
}

export interface StorageBufferBinding {
  group: number;
  binding: number;
  readOnly: boolean;
}

export interface UniformBufferBinding {
  group: number;
  binding: number;
  fields: UniformField[];
}

export type VertexSemantic =
  | 'position'
  | 'normal'
  | 'tangent'
  | 'uv0'
  | 'uv1'
  | 'uv2'
  | 'uv3'
  | 'color0'
  | 'color1'
  | 'joint-indices'
  | 'joint-weights';

export interface VertexAttributeSpec {
  location: number;
  semantic: VertexSemantic;
  format: VertexFormat;
}

export interface VertexAttributeDesc {
  format: VertexFormat;
  offset: number;
  shaderLocation: number;
}

// MARK: Material Types

export interface BlendComponentDesc {
  srcFactor: number; // BlendFactor enum
  dstFactor: number; // BlendFactor enum
  operation: number; // BlendOperation enum
}

export interface BlendStateDesc {
  color: BlendComponentDesc;
  alpha: BlendComponentDesc;
}

export interface StencilFaceStateDesc {
  compare: CompareFunction;
  failOp: number; // StencilOperation enum
  depthFailOp: number; // StencilOperation enum
  passOp: number; // StencilOperation enum
}

export interface StencilStateDesc {
  front: StencilFaceStateDesc;
  back: StencilFaceStateDesc;
  readMask: number;
  writeMask: number;
}

export interface DepthBiasStateDesc {
  constant: number;
  slopeScale: number;
  clamp: number;
}

export interface DepthStencilStateDesc {
  format: TextureFormat;
  depthWriteEnabled: boolean;
  depthCompare: CompareFunction;
  stencil: StencilStateDesc;
  bias: DepthBiasStateDesc;
}

export interface PrimitiveStateDesc {
  topology: number; // PrimitiveTopology enum
  stripIndexFormat?: IndexFormat;
  frontFace: number; // FrontFace enum
  cullMode?: number; // CullMode enum
  unclippedDepth: boolean;
  polygonMode: number; // PolygonMode enum
  conservative: boolean;
}

// MARK: Component Types

export type ViewportMode = 'relative' | 'absolute';

export interface Viewport {
  positionMode: ViewportMode;
  sizeMode: ViewportMode;
  x: number;
  y: number;
  width: number;
  height: number;
  anchor: [number, number]; // Vec2
}

// Use gl-matrix mat4 type (Float32Array of 16 elements)
import type { mat4 } from 'gl-matrix';
export type Mat4 = mat4;

// MARK: Shader Commands

export interface CmdShaderCreateArgs {
  shaderId: ShaderId;
  windowId: number;
  bufferId: number;
  label?: string;
  uniformBuffers: UniformBufferBinding[];
  textureBindings: TextureBinding[];
  storageBuffers: StorageBufferBinding[];
  vertexAttributes: VertexAttributeSpec[];
}

export interface CmdShaderDisposeArgs {
  shaderId: ShaderId;
  windowId: number;
}

// MARK: Geometry Commands

export interface CmdGeometryCreateArgs {
  geometryId: GeometryId;
  windowId: number;
  vertexBufferId: number;
  indexBufferId: number;
  vertexCount: number;
  indexCount: number;
  vertexAttributes: VertexAttributeDesc[];
  indexFormat: IndexFormat;
  label?: string;
}

export interface CmdGeometryDisposeArgs {
  geometryId: GeometryId;
  windowId: number;
}

// MARK: Material Commands

export interface CmdMaterialCreateArgs {
  materialId: MaterialId;
  windowId: number;
  shaderId: ShaderId;
  textures: TextureId[];
  blend?: BlendStateDesc;
  depthStencil?: DepthStencilStateDesc;
  primitive: PrimitiveStateDesc;
  label?: string;
}

export interface CmdMaterialUpdateArgs {
  materialId: MaterialId;
  windowId: number;
  shaderId?: ShaderId;
  textures?: TextureId[];
}

export interface CmdMaterialDisposeArgs {
  materialId: MaterialId;
  windowId: number;
}

// MARK: Texture Commands

export interface CmdTextureCreateArgs {
  textureId: TextureId;
  windowId: number;
  bufferId: number;
  width: number;
  height: number;
  format: TextureFormat;
  usage: TextureUsage[];
  mipLevelCount: number;
  label?: string;
}

export interface CmdTextureUpdateArgs {
  textureId: TextureId;
  windowId: number;
  bufferId: number;
  x: number;
  y: number;
  width: number;
  height: number;
  mipLevel: number;
}

export interface CmdTextureDisposeArgs {
  textureId: TextureId;
  windowId: number;
}

// MARK: Sampler Commands

export interface CmdSamplerCreateArgs {
  samplerId: SamplerId;
  windowId: number;
  addressModeU: AddressMode;
  addressModeV: AddressMode;
  addressModeW: AddressMode;
  magFilter: FilterMode;
  minFilter: FilterMode;
  mipmapFilter: MipmapFilterMode;
  lodMinClamp: number;
  lodMaxClamp: number;
  compare?: CompareFunction;
  anisotropyClamp: number;
  borderColor?: BorderColor;
  label?: string;
}

export interface CmdSamplerUpdateArgs {
  samplerId: SamplerId;
  windowId: number;
  addressModeU: AddressMode;
  addressModeV: AddressMode;
  addressModeW: AddressMode;
  magFilter: FilterMode;
  minFilter: FilterMode;
  mipmapFilter: MipmapFilterMode;
  lodMinClamp: number;
  lodMaxClamp: number;
  compare?: CompareFunction;
  anisotropyClamp: number;
  borderColor?: BorderColor;
  label?: string;
}

export interface CmdSamplerDisposeArgs {
  samplerId: SamplerId;
  windowId: number;
}

// MARK: Camera Commands

export interface CmdCameraCreateArgs {
  componentId: ComponentId;
  windowId: number;
  projMat: Mat4;
  viewMat: Mat4;
  viewport: Viewport;
  layerMask?: number;
}

export interface CmdCameraUpdateArgs {
  componentId: ComponentId;
  windowId: number;
  projMat?: Mat4;
  viewMat?: Mat4;
  viewport?: Viewport;
  layerMask?: number;
}

export interface CmdCameraDisposeArgs {
  componentId: ComponentId;
  windowId: number;
}

// MARK: Model Commands

export interface CmdModelCreateArgs {
  componentId: ComponentId;
  windowId: number;
  geometryId: GeometryId;
  materialId: MaterialId;
  modelMat: Mat4;
  layerMask?: number;
}

export interface CmdModelUpdateArgs {
  componentId: ComponentId;
  windowId: number;
  geometryId?: GeometryId;
  materialId?: MaterialId;
  modelMat?: Mat4;
  layerMask?: number;
}

export interface CmdModelDisposeArgs {
  componentId: ComponentId;
  windowId: number;
}

// MARK: Render Command Union

export type RenderCmd =
  | { type: 'cmd-shader-create'; content: CmdShaderCreateArgs }
  | { type: 'cmd-shader-dispose'; content: CmdShaderDisposeArgs }
  | { type: 'cmd-geometry-create'; content: CmdGeometryCreateArgs }
  | { type: 'cmd-geometry-dispose'; content: CmdGeometryDisposeArgs }
  | { type: 'cmd-material-create'; content: CmdMaterialCreateArgs }
  | { type: 'cmd-material-update'; content: CmdMaterialUpdateArgs }
  | { type: 'cmd-material-dispose'; content: CmdMaterialDisposeArgs }
  | { type: 'cmd-texture-create'; content: CmdTextureCreateArgs }
  | { type: 'cmd-texture-update'; content: CmdTextureUpdateArgs }
  | { type: 'cmd-texture-dispose'; content: CmdTextureDisposeArgs }
  | { type: 'cmd-sampler-create'; content: CmdSamplerCreateArgs }
  | { type: 'cmd-sampler-update'; content: CmdSamplerUpdateArgs }
  | { type: 'cmd-sampler-dispose'; content: CmdSamplerDisposeArgs }
  | { type: 'cmd-camera-create'; content: CmdCameraCreateArgs }
  | { type: 'cmd-camera-update'; content: CmdCameraUpdateArgs }
  | { type: 'cmd-camera-dispose'; content: CmdCameraDisposeArgs }
  | { type: 'cmd-model-create'; content: CmdModelCreateArgs }
  | { type: 'cmd-model-update'; content: CmdModelUpdateArgs }
  | { type: 'cmd-model-dispose'; content: CmdModelDisposeArgs };
