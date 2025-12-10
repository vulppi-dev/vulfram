import type { Viewport } from './render';

/**
 * Create a fullscreen viewport (covers entire window)
 */
export function createFullscreenViewport(): Viewport {
  return {
    positionMode: 'relative',
    sizeMode: 'relative',
    x: 0,
    y: 0,
    width: 1,
    height: 1,
    anchor: [0, 0],
  };
}

/**
 * Create a viewport with relative position and absolute size
 * Useful for UI elements that need fixed pixel sizes but relative positioning
 *
 * @param x - Relative X position (0.0 to 1.0)
 * @param y - Relative Y position (0.0 to 1.0)
 * @param width - Absolute width in pixels
 * @param height - Absolute height in pixels
 * @param anchorX - Anchor X (0.0 = left, 0.5 = center, 1.0 = right)
 * @param anchorY - Anchor Y (0.0 = top, 0.5 = center, 1.0 = bottom)
 */
export function createRelativePosAbsoluteSizeViewport(
  x: number,
  y: number,
  width: number,
  height: number,
  anchorX: number = 0,
  anchorY: number = 0,
): Viewport {
  return {
    positionMode: 'relative',
    sizeMode: 'absolute',
    x,
    y,
    width,
    height,
    anchor: [anchorX, anchorY],
  };
}

/**
 * Create a viewport with absolute position and relative size
 * Useful for margins or offsets with scalable content
 *
 * @param x - Absolute X position in pixels
 * @param y - Absolute Y position in pixels
 * @param width - Relative width (0.0 to 1.0)
 * @param height - Relative height (0.0 to 1.0)
 * @param anchorX - Anchor X (0.0 = left, 0.5 = center, 1.0 = right)
 * @param anchorY - Anchor Y (0.0 = top, 0.5 = center, 1.0 = bottom)
 */
export function createAbsolutePosRelativeSizeViewport(
  x: number,
  y: number,
  width: number,
  height: number,
  anchorX: number = 0,
  anchorY: number = 0,
): Viewport {
  return {
    positionMode: 'absolute',
    sizeMode: 'relative',
    x,
    y,
    width,
    height,
    anchor: [anchorX, anchorY],
  };
}

/**
 * Create a centered viewport with fixed size
 *
 * @param width - Width in pixels
 * @param height - Height in pixels
 */
export function createCenteredAbsoluteViewport(
  width: number,
  height: number,
): Viewport {
  return {
    positionMode: 'relative',
    sizeMode: 'absolute',
    x: 0.5,
    y: 0.5,
    width,
    height,
    anchor: [0.5, 0.5], // Center anchor
  };
}

/**
 * Create a viewport anchored to a corner
 *
 * @param corner - 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'
 * @param width - Width (relative or absolute based on isAbsolute)
 * @param height - Height (relative or absolute based on isAbsolute)
 * @param isAbsolute - If true, width/height are in pixels; if false, they're 0.0-1.0
 * @param offsetX - Offset from corner in pixels
 * @param offsetY - Offset from corner in pixels
 */
export function createCornerViewport(
  corner: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right',
  width: number,
  height: number,
  isAbsolute: boolean = true,
  offsetX: number = 0,
  offsetY: number = 0,
): Viewport {
  const sizeMode = isAbsolute ? 'absolute' : 'relative';

  switch (corner) {
    case 'top-left':
      return {
        positionMode: 'absolute',
        sizeMode,
        x: offsetX,
        y: offsetY,
        width,
        height,
        anchor: [0, 0],
      };
    case 'top-right':
      return {
        positionMode: 'relative',
        sizeMode,
        x: 1.0,
        y: 0,
        width,
        height,
        anchor: [1, 0],
      };
    case 'bottom-left':
      return {
        positionMode: 'relative',
        sizeMode,
        x: 0,
        y: 1.0,
        width,
        height,
        anchor: [0, 1],
      };
    case 'bottom-right':
      return {
        positionMode: 'relative',
        sizeMode,
        x: 1.0,
        y: 1.0,
        width,
        height,
        anchor: [1, 1],
      };
  }
}

/**
 * Create a viewport with margin (in pixels or relative)
 *
 * @param margin - Margin size
 * @param isAbsolute - If true, margin is in pixels; if false, it's relative (0.0-1.0)
 */
export function createViewportWithMargin(
  margin: number,
  isAbsolute: boolean = true,
): Viewport {
  if (isAbsolute) {
    return {
      positionMode: 'absolute',
      sizeMode: 'relative',
      x: margin,
      y: margin,
      width: 1.0,
      height: 1.0,
      anchor: [0, 0],
    };
  } else {
    return {
      positionMode: 'relative',
      sizeMode: 'relative',
      x: margin,
      y: margin,
      width: 1.0 - margin * 2,
      height: 1.0 - margin * 2,
      anchor: [0, 0],
    };
  }
}

/**
 * Create a split viewport (horizontal or vertical)
 *
 * @param split - Split position (0.0 to 1.0)
 * @param side - Which side to create viewport for
 * @param direction - 'horizontal' or 'vertical'
 */
export function createSplitViewport(
  split: number,
  side: 'first' | 'second',
  direction: 'horizontal' | 'vertical' = 'horizontal',
): Viewport {
  if (direction === 'horizontal') {
    // Horizontal split (left/right)
    return {
      positionMode: 'relative',
      sizeMode: 'relative',
      x: side === 'first' ? 0 : split,
      y: 0,
      width: side === 'first' ? split : 1.0 - split,
      height: 1.0,
      anchor: [0, 0],
    };
  } else {
    // Vertical split (top/bottom)
    return {
      positionMode: 'relative',
      sizeMode: 'relative',
      x: 0,
      y: side === 'first' ? 0 : split,
      width: 1.0,
      height: side === 'first' ? split : 1.0 - split,
      anchor: [0, 0],
    };
  }
}
