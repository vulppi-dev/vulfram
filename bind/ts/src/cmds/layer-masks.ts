/**
 * Layer mask constants and utilities for visibility control
 *
 * Layer masks use bitwise operations to control which cameras can see which models.
 * Each bit in the 32-bit mask represents a layer (0-31).
 *
 * Visibility rule: `(cameraLayerMask & modelLayerMask) > 0`
 *
 * @example
 * ```typescript
 * // Camera that sees everything
 * const camera = { layerMask: LAYER_ALL };
 *
 * // Model only visible to UI cameras
 * const uiElement = { layerMask: LAYER_UI };
 *
 * // Camera that sees world and effects
 * const mainCamera = { layerMask: LAYER_WORLD | LAYER_EFFECTS };
 *
 * // Model visible on multiple layers
 * const model = { layerMask: LAYER_WORLD | LAYER_MINIMAP };
 * ```
 */

// MARK: Predefined Layers

/** Layer 0: Default world geometry */
export const LAYER_WORLD = 1 << 0;

/** Layer 1: UI elements */
export const LAYER_UI = 1 << 1;

/** Layer 2: Debug/gizmos (editor only) */
export const LAYER_DEBUG = 1 << 2;

/** Layer 3: Effects/particles */
export const LAYER_EFFECTS = 1 << 3;

/** Layer 4: Minimap rendering */
export const LAYER_MINIMAP = 1 << 4;

/** Layer 5: Shadow casters */
export const LAYER_SHADOW = 1 << 5;

/** Layer 6: Water/transparent surfaces */
export const LAYER_WATER = 1 << 6;

/** Layer 7: Sky/background */
export const LAYER_SKY = 1 << 7;

/** Layer 8: Player/character */
export const LAYER_PLAYER = 1 << 8;

/** Layer 9: NPCs */
export const LAYER_NPC = 1 << 9;

/** Layer 10: Enemies */
export const LAYER_ENEMY = 1 << 10;

/** Layer 11: Props/interactables */
export const LAYER_PROPS = 1 << 11;

/** Layer 12: Terrain */
export const LAYER_TERRAIN = 1 << 12;

/** Layer 13: Foliage/vegetation */
export const LAYER_FOLIAGE = 1 << 13;

/** Layer 14: Buildings/structures */
export const LAYER_BUILDINGS = 1 << 14;

/** Layer 15: Overlay UI (tooltips, cursor, etc.) */
export const LAYER_OVERLAY = 1 << 15;

/** All layers enabled */
export const LAYER_ALL = 0xffffffff;

/** No layers enabled */
export const LAYER_NONE = 0;

// MARK: Layer Utilities

/**
 * Create a custom layer mask from layer indices
 *
 * @param layers - Array of layer indices (0-31)
 * @returns Combined layer mask
 *
 * @example
 * ```typescript
 * const mask = createLayerMask(0, 3, 7); // Layers 0, 3, and 7
 * ```
 */
export function createLayerMask(...layers: number[]): number {
  return layers.reduce((mask, layer) => mask | (1 << layer), 0);
}

/**
 * Add layers to an existing mask
 *
 * @param mask - Existing layer mask
 * @param layers - Layers to add
 * @returns Updated layer mask
 *
 * @example
 * ```typescript
 * let mask = LAYER_WORLD;
 * mask = addLayers(mask, LAYER_UI, LAYER_DEBUG);
 * ```
 */
export function addLayers(mask: number, ...layers: number[]): number {
  return layers.reduce((m, layer) => m | layer, mask);
}

/**
 * Remove layers from an existing mask
 *
 * @param mask - Existing layer mask
 * @param layers - Layers to remove
 * @returns Updated layer mask
 *
 * @example
 * ```typescript
 * let mask = LAYER_ALL;
 * mask = removeLayers(mask, LAYER_DEBUG, LAYER_OVERLAY);
 * ```
 */
export function removeLayers(mask: number, ...layers: number[]): number {
  return layers.reduce((m, layer) => m & ~layer, mask);
}

/**
 * Toggle layers in an existing mask
 *
 * @param mask - Existing layer mask
 * @param layers - Layers to toggle
 * @returns Updated layer mask
 */
export function toggleLayers(mask: number, ...layers: number[]): number {
  return layers.reduce((m, layer) => m ^ layer, mask);
}

/**
 * Check if a mask contains specific layers
 *
 * @param mask - Layer mask to check
 * @param layers - Layers to check for
 * @returns True if mask contains all specified layers
 */
export function hasLayers(mask: number, ...layers: number[]): boolean {
  return layers.every((layer) => (mask & layer) !== 0);
}

/**
 * Check if a mask contains any of the specified layers
 *
 * @param mask - Layer mask to check
 * @param layers - Layers to check for
 * @returns True if mask contains at least one of the specified layers
 */
export function hasAnyLayer(mask: number, ...layers: number[]): boolean {
  return layers.some((layer) => (mask & layer) !== 0);
}

/**
 * Check if two masks would result in visible rendering
 * (i.e., camera can see model)
 *
 * @param cameraLayerMask - Camera's layer mask
 * @param modelLayerMask - Model's layer mask
 * @returns True if camera can see model
 */
export function isVisible(
  cameraLayerMask: number,
  modelLayerMask: number,
): boolean {
  return (cameraLayerMask & modelLayerMask) > 0;
}

/**
 * Get list of active layer indices from a mask
 *
 * @param mask - Layer mask
 * @returns Array of active layer indices (0-31)
 *
 * @example
 * ```typescript
 * const layers = getActiveLayers(LAYER_WORLD | LAYER_UI);
 * // Returns: [0, 1]
 * ```
 */
export function getActiveLayers(mask: number): number[] {
  const layers: number[] = [];
  for (let i = 0; i < 32; i++) {
    if ((mask & (1 << i)) !== 0) {
      layers.push(i);
    }
  }
  return layers;
}

// MARK: Preset Masks

/** Preset: Main game camera (world + effects + characters) */
export const MASK_MAIN_CAMERA =
  LAYER_WORLD |
  LAYER_EFFECTS |
  LAYER_PLAYER |
  LAYER_NPC |
  LAYER_ENEMY |
  LAYER_PROPS |
  LAYER_TERRAIN |
  LAYER_FOLIAGE |
  LAYER_BUILDINGS |
  LAYER_WATER |
  LAYER_SKY;

/** Preset: UI camera (only UI and overlay) */
export const MASK_UI_CAMERA = LAYER_UI | LAYER_OVERLAY;

/** Preset: Shadow map camera (shadow casters only) */
export const MASK_SHADOW_CAMERA =
  LAYER_SHADOW | LAYER_WORLD | LAYER_PLAYER | LAYER_NPC | LAYER_ENEMY;

/** Preset: Minimap camera */
export const MASK_MINIMAP_CAMERA =
  LAYER_MINIMAP | LAYER_WORLD | LAYER_PLAYER | LAYER_NPC | LAYER_ENEMY;

/** Preset: Debug camera (everything including debug) */
export const MASK_DEBUG_CAMERA = LAYER_ALL;
