export enum VulframResult {
  Success = 0,
  UnknownError = 1,
  NotInitialized,
  AlreadyInitialized,
  WrongThread,
  CmdInvalidMessagePackError,
  BufferNotFound,
  BufferIdCollision,
  InvalidUploadType,
}

// MARK: Texture Enums

export enum TextureFormat {
  // 8-bit formats
  R8Unorm = 0,
  R8Snorm = 1,
  R8Uint = 2,
  R8Sint = 3,
  // 16-bit formats
  R16Uint = 4,
  R16Sint = 5,
  R16Float = 6,
  Rg8Unorm = 7,
  Rg8Snorm = 8,
  Rg8Uint = 9,
  Rg8Sint = 10,
  // 32-bit formats
  R32Uint = 11,
  R32Sint = 12,
  R32Float = 13,
  Rg16Uint = 14,
  Rg16Sint = 15,
  Rg16Float = 16,
  Rgba8Unorm = 17,
  Rgba8UnormSrgb = 18,
  Rgba8Snorm = 19,
  Rgba8Uint = 20,
  Rgba8Sint = 21,
  Bgra8Unorm = 22,
  Bgra8UnormSrgb = 23,
  // Packed 32-bit formats
  Rgb10a2Unorm = 24,
  // 64-bit formats
  Rg32Uint = 25,
  Rg32Sint = 26,
  Rg32Float = 27,
  Rgba16Uint = 28,
  Rgba16Sint = 29,
  Rgba16Float = 30,
  // 128-bit formats
  Rgba32Uint = 31,
  Rgba32Sint = 32,
  Rgba32Float = 33,
  // Depth/stencil formats
  Depth32Float = 34,
  Depth24Plus = 35,
  Depth24PlusStencil8 = 36,
  Depth32FloatStencil8 = 37,
}

export enum TextureUsage {
  CopySrc = 1,
  CopyDst = 2,
  TextureBinding = 4,
  StorageBinding = 8,
  RenderAttachment = 16,
}

// MARK: Vertex Format Enum

export enum VertexFormat {
  Uint8x2 = 0,
  Uint8x4 = 1,
  Sint8x2 = 2,
  Sint8x4 = 3,
  Unorm8x2 = 4,
  Unorm8x4 = 5,
  Snorm8x2 = 6,
  Snorm8x4 = 7,
  Uint16x2 = 8,
  Uint16x4 = 9,
  Sint16x2 = 10,
  Sint16x4 = 11,
  Unorm16x2 = 12,
  Unorm16x4 = 13,
  Snorm16x2 = 14,
  Snorm16x4 = 15,
  Float16x2 = 16,
  Float16x4 = 17,
  Float32 = 18,
  Float32x2 = 19,
  Float32x3 = 20,
  Float32x4 = 21,
  Uint32 = 22,
  Uint32x2 = 23,
  Uint32x3 = 24,
  Uint32x4 = 25,
  Sint32 = 26,
  Sint32x2 = 27,
  Sint32x3 = 28,
  Sint32x4 = 29,
  Float64 = 30,
  Float64x2 = 31,
  Float64x3 = 32,
  Float64x4 = 33,
}

export enum IndexFormat {
  Uint16 = 0,
  Uint32 = 1,
}

// MARK: Material Enums

export enum BlendFactor {
  Zero = 0,
  One = 1,
  Src = 2,
  OneMinusSrc = 3,
  SrcAlpha = 4,
  OneMinusSrcAlpha = 5,
  Dst = 6,
  OneMinusDst = 7,
  DstAlpha = 8,
  OneMinusDstAlpha = 9,
  SrcAlphaSaturated = 10,
  Constant = 11,
  OneMinusConstant = 12,
}

export enum BlendOperation {
  Add = 0,
  Subtract = 1,
  ReverseSubtract = 2,
  Min = 3,
  Max = 4,
}

export enum CompareFunction {
  Never = 0,
  Less = 1,
  Equal = 2,
  LessEqual = 3,
  Greater = 4,
  NotEqual = 5,
  GreaterEqual = 6,
  Always = 7,
}

export enum StencilOperation {
  Keep = 0,
  Zero = 1,
  Replace = 2,
  Invert = 3,
  IncrementClamp = 4,
  DecrementClamp = 5,
  IncrementWrap = 6,
  DecrementWrap = 7,
}

export enum PrimitiveTopology {
  PointList = 0,
  LineList = 1,
  LineStrip = 2,
  TriangleList = 3,
  TriangleStrip = 4,
}

export enum FrontFace {
  Ccw = 0,
  Cw = 1,
}

export enum CullMode {
  Front = 0,
  Back = 1,
}

export enum PolygonMode {
  Fill = 0,
  Line = 1,
  Point = 2,
}

// MARK: Sampler Enums

export enum AddressMode {
  ClampToEdge = 0,
  Repeat = 1,
  MirrorRepeat = 2,
  ClampToBorder = 3,
}

export enum FilterMode {
  Nearest = 0,
  Linear = 1,
}

export enum MipmapFilterMode {
  Nearest = 0,
  Linear = 1,
}

export enum BorderColor {
  TransparentBlack = 0,
  OpaqueBlack = 1,
  OpaqueWhite = 2,
}

// MARK: Upload Type

/**
 * Upload type - defines the purpose of the buffer data
 */
export enum UploadType {
  /** Raw binary data (default) */
  Raw = 0,
  /** Shader source code (WGSL, GLSL, SPIR-V) */
  ShaderSource = 1,
  /** Vertex data for geometry */
  VertexData = 2,
  /** Index data for geometry */
  IndexData = 3,
  /** Image data (PNG, JPEG, WebP, AVIF) - will be decoded when consumed */
  ImageData = 4,
  /** Generic binary asset */
  BinaryAsset = 5,
}

// MARK: Uniform Types

export enum UniformType {
  Float = 0,
  Int = 1,
  UInt = 2,
  Bool = 3,
  Vec2 = 4,
  Vec3 = 5,
  Vec4 = 6,
  Vec2i = 7,
  Vec3i = 8,
  Vec4i = 9,
  Vec2u = 10,
  Vec3u = 11,
  Vec4u = 12,
  Mat2x2 = 13,
  Mat2x3 = 14,
  Mat2x4 = 15,
  Mat3x2 = 16,
  Mat3x3 = 17,
  Mat3x4 = 18,
  Mat4x2 = 19,
  Mat4x3 = 20,
  Mat4x4 = 21,
  AtomicInt = 22,
  AtomicUInt = 23,
}

// MARK: Texture Binding Types

export enum TextureSampleType {
  Float = 0,
  Depth = 1,
  Sint = 2,
  Uint = 3,
}

export enum TextureViewDimension {
  D1 = 0,
  D2 = 1,
  D2Array = 2,
  Cube = 3,
  CubeArray = 4,
  D3 = 5,
}

export enum SamplerBindingType {
  Filtering = 0,
  NonFiltering = 1,
  Comparison = 2,
}

// MARK: Window State

export enum WindowState {
  Minimized = 0,
  Maximized = 1,
  Windowed = 2,
  Fullscreen = 3,
  WindowedFullscreen = 4,
}

// MARK: Vertex Semantics

export enum VertexSemantic {
  Position = 0,
  Normal = 1,
  Tangent = 2,
  UV0 = 3,
  UV1 = 4,
  UV2 = 5,
  UV3 = 6,
  Color0 = 7,
  Color1 = 8,
  JointIndices = 9,
  JointWeights = 10,
}

// MARK: Viewport Mode

export enum ViewportMode {
  Absolute = 0,
  Relative = 1,
}
