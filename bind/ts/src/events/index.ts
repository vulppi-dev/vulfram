// Re-export all event types
export * from './common';
export * from './window';
export * from './pointer';
export * from './keyboard';
export * from './gamepad';
export * from './system';

import type { WindowEvent } from './window';
import type { PointerEvent } from './pointer';
import type { KeyboardEvent } from './keyboard';
import type { GamepadEvent } from './gamepad';
import type { SystemEvent } from './system';

// MARK: Engine Event

export type EngineEvent =
  | { type: 'window'; content: WindowEvent }
  | { type: 'pointer'; content: PointerEvent }
  | { type: 'keyboard'; content: KeyboardEvent }
  | { type: 'gamepad'; content: GamepadEvent }
  | { type: 'system'; content: SystemEvent };

// MARK: Type Guards

export function isWindowEvent(
  event: EngineEvent,
): event is { type: 'window'; content: WindowEvent } {
  return event.type === 'window';
}

export function isPointerEvent(
  event: EngineEvent,
): event is { type: 'pointer'; content: PointerEvent } {
  return event.type === 'pointer';
}

export function isKeyboardEvent(
  event: EngineEvent,
): event is { type: 'keyboard'; content: KeyboardEvent } {
  return event.type === 'keyboard';
}

export function isGamepadEvent(
  event: EngineEvent,
): event is { type: 'gamepad'; content: GamepadEvent } {
  return event.type === 'gamepad';
}

export function isSystemEvent(
  event: EngineEvent,
): event is { type: 'system'; content: SystemEvent } {
  return event.type === 'system';
}
