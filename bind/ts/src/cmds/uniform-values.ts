/**
 * Uniform value types for shader uniform buffers
 *
 * These types are serialized to MessagePack and sent to the engine core.
 * They match the Rust `UniformValue` enum.
 */

// MARK: Vector Types

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Vec4 = [number, number, number, number];

export type IVec2 = [number, number];
export type IVec3 = [number, number, number];
export type IVec4 = [number, number, number, number];

export type UVec2 = [number, number];
export type UVec3 = [number, number, number];
export type UVec4 = [number, number, number, number];

// MARK: Matrix Types

export type Mat2 = [number, number, number, number];

export type Mat3 = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
];

// Mat4 is already defined in render.ts
import type { Mat4 } from './render';
export type { Mat4 };

// MARK: Uniform Value Union

export type UniformValue =
  // Scalars
  | { type: 'float'; value: number }
  | { type: 'int'; value: number }
  | { type: 'u-int'; value: number }
  | { type: 'bool'; value: boolean }
  // Vectors
  | { type: 'vec2'; value: Vec2 }
  | { type: 'vec3'; value: Vec3 }
  | { type: 'vec4'; value: Vec4 }
  | { type: 'vec2i'; value: IVec2 }
  | { type: 'vec3i'; value: IVec3 }
  | { type: 'vec4i'; value: IVec4 }
  | { type: 'vec2u'; value: UVec2 }
  | { type: 'vec3u'; value: UVec3 }
  | { type: 'vec4u'; value: UVec4 }
  // Matrices
  | { type: 'mat2'; value: Mat2 }
  | { type: 'mat3'; value: Mat3 }
  | { type: 'mat4'; value: Mat4 }
  // Arrays
  | { type: 'float-array'; value: number[] }
  | { type: 'vec4-array'; value: Vec4[] }
  | { type: 'mat4-array'; value: Mat4[] };

// MARK: Uniform Value Helpers

/**
 * Create a float uniform value
 */
export function uniformFloat(value: number): UniformValue {
  return { type: 'float', value };
}

/**
 * Create an int uniform value
 */
export function uniformInt(value: number): UniformValue {
  return { type: 'int', value };
}

/**
 * Create a uint uniform value
 */
export function uniformUInt(value: number): UniformValue {
  return { type: 'u-int', value };
}

/**
 * Create a bool uniform value
 */
export function uniformBool(value: boolean): UniformValue {
  return { type: 'bool', value };
}

/**
 * Create a vec2 uniform value
 */
export function uniformVec2(x: number, y: number): UniformValue {
  return { type: 'vec2', value: [x, y] };
}

/**
 * Create a vec3 uniform value
 */
export function uniformVec3(x: number, y: number, z: number): UniformValue {
  return { type: 'vec3', value: [x, y, z] };
}

/**
 * Create a vec4 uniform value
 */
export function uniformVec4(
  x: number,
  y: number,
  z: number,
  w: number,
): UniformValue {
  return { type: 'vec4', value: [x, y, z, w] };
}

/**
 * Create a vec2i uniform value
 */
export function uniformVec2i(x: number, y: number): UniformValue {
  return { type: 'vec2i', value: [x, y] };
}

/**
 * Create a vec3i uniform value
 */
export function uniformVec3i(x: number, y: number, z: number): UniformValue {
  return { type: 'vec3i', value: [x, y, z] };
}

/**
 * Create a vec4i uniform value
 */
export function uniformVec4i(
  x: number,
  y: number,
  z: number,
  w: number,
): UniformValue {
  return { type: 'vec4i', value: [x, y, z, w] };
}

/**
 * Create a vec2u uniform value
 */
export function uniformVec2u(x: number, y: number): UniformValue {
  return { type: 'vec2u', value: [x, y] };
}

/**
 * Create a vec3u uniform value
 */
export function uniformVec3u(x: number, y: number, z: number): UniformValue {
  return { type: 'vec3u', value: [x, y, z] };
}

/**
 * Create a vec4u uniform value
 */
export function uniformVec4u(
  x: number,
  y: number,
  z: number,
  w: number,
): UniformValue {
  return { type: 'vec4u', value: [x, y, z, w] };
}

/**
 * Create a mat2 uniform value
 */
export function uniformMat2(value: Mat2): UniformValue {
  return { type: 'mat2', value };
}

/**
 * Create a mat3 uniform value
 */
export function uniformMat3(value: Mat3): UniformValue {
  return { type: 'mat3', value };
}

/**
 * Create a mat4 uniform value
 */
export function uniformMat4(value: Mat4): UniformValue {
  return { type: 'mat4', value };
}

/**
 * Create a float array uniform value
 */
export function uniformFloatArray(value: number[]): UniformValue {
  return { type: 'float-array', value };
}

/**
 * Create a vec4 array uniform value
 */
export function uniformVec4Array(value: Vec4[]): UniformValue {
  return { type: 'vec4-array', value };
}

/**
 * Create a mat4 array uniform value
 */
export function uniformMat4Array(value: Mat4[]): UniformValue {
  return { type: 'mat4-array', value };
}

// MARK: Color Helpers

/**
 * Create a color uniform value (vec4) from RGBA values (0-1 range)
 */
export function uniformColor(
  r: number,
  g: number,
  b: number,
  a: number = 1,
): UniformValue {
  return uniformVec4(r, g, b, a);
}

/**
 * Create a color uniform value (vec4) from RGB 0-255 integers
 */
export function uniformColorRGB255(
  r: number,
  g: number,
  b: number,
  a: number = 255,
): UniformValue {
  return uniformVec4(r / 255, g / 255, b / 255, a / 255);
}

/**
 * Create a color uniform value (vec4) from hex string
 *
 * @param hex - Hex color string (e.g., '#FF0000', '#FF0000FF', 'FF0000')
 */
export function uniformColorHex(hex: string): UniformValue {
  // Remove # if present
  hex = hex.replace('#', '');

  // Parse RGB or RGBA
  let r: number, g: number, b: number, a: number;

  if (hex.length === 6) {
    // RGB
    r = parseInt(hex.substring(0, 2), 16) / 255;
    g = parseInt(hex.substring(2, 4), 16) / 255;
    b = parseInt(hex.substring(4, 6), 16) / 255;
    a = 1;
  } else if (hex.length === 8) {
    // RGBA
    r = parseInt(hex.substring(0, 2), 16) / 255;
    g = parseInt(hex.substring(2, 4), 16) / 255;
    b = parseInt(hex.substring(4, 6), 16) / 255;
    a = parseInt(hex.substring(6, 8), 16) / 255;
  } else {
    throw new Error(`Invalid hex color: ${hex}`);
  }

  return uniformVec4(r, g, b, a);
}

// MARK: Common Colors (as helper functions)

export const uniformColorWhite = () => uniformColor(1, 1, 1, 1);
export const uniformColorBlack = () => uniformColor(0, 0, 0, 1);
export const uniformColorRed = () => uniformColor(1, 0, 0, 1);
export const uniformColorGreen = () => uniformColor(0, 1, 0, 1);
export const uniformColorBlue = () => uniformColor(0, 0, 1, 1);
export const uniformColorYellow = () => uniformColor(1, 1, 0, 1);
export const uniformColorCyan = () => uniformColor(0, 1, 1, 1);
export const uniformColorMagenta = () => uniformColor(1, 0, 1, 1);
export const uniformColorTransparent = () => uniformColor(0, 0, 0, 0);
