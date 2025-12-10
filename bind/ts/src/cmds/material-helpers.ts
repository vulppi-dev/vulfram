import {
  BlendFactor,
  BlendOperation,
  CompareFunction,
  StencilOperation,
  PrimitiveTopology,
  FrontFace,
  CullMode,
  PolygonMode,
  IndexFormat,
} from '../enums';
import type {
  BlendComponentDesc,
  BlendStateDesc,
  StencilFaceStateDesc,
  StencilStateDesc,
  DepthBiasStateDesc,
  DepthStencilStateDesc,
  PrimitiveStateDesc,
} from './render';

// MARK: Blend State Helpers

export function createBlendComponent(
  srcFactor: BlendFactor = BlendFactor.One,
  dstFactor: BlendFactor = BlendFactor.Zero,
  operation: BlendOperation = BlendOperation.Add,
): BlendComponentDesc {
  return {
    srcFactor,
    dstFactor,
    operation,
  };
}

export function createBlendState(
  color?: BlendComponentDesc,
  alpha?: BlendComponentDesc,
): BlendStateDesc {
  return {
    color: color || createBlendComponent(),
    alpha: alpha || createBlendComponent(),
  };
}

/**
 * Create alpha blending preset (common for transparent objects)
 * Result = srcAlpha * src + (1 - srcAlpha) * dst
 */
export function createAlphaBlending(): BlendStateDesc {
  return {
    color: createBlendComponent(
      BlendFactor.SrcAlpha,
      BlendFactor.OneMinusSrcAlpha,
      BlendOperation.Add,
    ),
    alpha: createBlendComponent(
      BlendFactor.One,
      BlendFactor.OneMinusSrcAlpha,
      BlendOperation.Add,
    ),
  };
}

/**
 * Create additive blending preset (common for particles/effects)
 * Result = src + dst
 */
export function createAdditiveBlending(): BlendStateDesc {
  return {
    color: createBlendComponent(
      BlendFactor.One,
      BlendFactor.One,
      BlendOperation.Add,
    ),
    alpha: createBlendComponent(
      BlendFactor.One,
      BlendFactor.One,
      BlendOperation.Add,
    ),
  };
}

/**
 * Create multiplicative blending preset
 * Result = src * dst
 */
export function createMultiplicativeBlending(): BlendStateDesc {
  return {
    color: createBlendComponent(
      BlendFactor.Dst,
      BlendFactor.Zero,
      BlendOperation.Add,
    ),
    alpha: createBlendComponent(
      BlendFactor.Dst,
      BlendFactor.Zero,
      BlendOperation.Add,
    ),
  };
}

// MARK: Depth Stencil Helpers

export function createStencilFaceState(
  compare: CompareFunction = CompareFunction.Always,
  failOp: StencilOperation = StencilOperation.Keep,
  depthFailOp: StencilOperation = StencilOperation.Keep,
  passOp: StencilOperation = StencilOperation.Keep,
): StencilFaceStateDesc {
  return {
    compare,
    failOp,
    depthFailOp,
    passOp,
  };
}

export function createStencilState(
  front?: StencilFaceStateDesc,
  back?: StencilFaceStateDesc,
  readMask: number = 0xff,
  writeMask: number = 0xff,
): StencilStateDesc {
  return {
    front: front || createStencilFaceState(),
    back: back || createStencilFaceState(),
    readMask,
    writeMask,
  };
}

export function createDepthBiasState(
  constant: number = 0,
  slopeScale: number = 0,
  clamp: number = 0,
): DepthBiasStateDesc {
  return {
    constant,
    slopeScale,
    clamp,
  };
}

export function createDepthStencilState(
  depthCompare: CompareFunction = CompareFunction.Less,
  depthWriteEnabled: boolean = true,
  stencil?: StencilStateDesc,
  bias?: DepthBiasStateDesc,
): DepthStencilStateDesc {
  return {
    format: 35, // TextureFormat.Depth24Plus
    depthWriteEnabled,
    depthCompare,
    stencil: stencil || createStencilState(),
    bias: bias || createDepthBiasState(),
  };
}

// MARK: Primitive State Helpers

export function createPrimitiveState(
  topology: PrimitiveTopology = PrimitiveTopology.TriangleList,
  frontFace: FrontFace = FrontFace.Ccw,
  cullMode: CullMode | null = CullMode.Back,
  polygonMode: PolygonMode = PolygonMode.Fill,
  stripIndexFormat?: IndexFormat,
  unclippedDepth: boolean = false,
  conservative: boolean = false,
): PrimitiveStateDesc {
  return {
    topology,
    stripIndexFormat,
    frontFace,
    cullMode: cullMode !== null ? cullMode : undefined,
    unclippedDepth,
    polygonMode,
    conservative,
  };
}

/**
 * Create default opaque primitive state (triangle list, CCW, backface culling)
 */
export function createDefaultPrimitiveState(): PrimitiveStateDesc {
  return createPrimitiveState();
}

/**
 * Create wireframe primitive state (line mode rendering)
 */
export function createWireframePrimitiveState(): PrimitiveStateDesc {
  return createPrimitiveState(
    PrimitiveTopology.TriangleList,
    FrontFace.Ccw,
    null, // No culling for wireframe
    PolygonMode.Line,
  );
}

/**
 * Create double-sided primitive state (no culling)
 */
export function createDoubleSidedPrimitiveState(): PrimitiveStateDesc {
  return createPrimitiveState(
    PrimitiveTopology.TriangleList,
    FrontFace.Ccw,
    null, // No culling
  );
}
