import type { ElementState } from './common';

// MARK: Gamepad Events

/** Gamepad button types following standard gamepad mapping */
export type GamepadButton =
  // Face buttons
  | 'south' // A / Cross
  | 'east' // B / Circle
  | 'west' // X / Square
  | 'north' // Y / Triangle
  // Shoulder buttons
  | 'left-bumper'
  | 'right-bumper'
  | 'left-trigger'
  | 'right-trigger'
  // Center buttons
  | 'select'
  | 'start'
  | 'mode' // Guide / Home
  // Stick buttons
  | 'left-stick'
  | 'right-stick'
  // D-pad
  | 'dpad-up'
  | 'dpad-down'
  | 'dpad-left'
  | 'dpad-right'
  // Other
  | { other: number };

/** Gamepad axis types */
export type GamepadAxis =
  | 'left-stick-x'
  | 'left-stick-y'
  | 'right-stick-x'
  | 'right-stick-y'
  | 'left-trigger'
  | 'right-trigger'
  | { other: number };

export interface GamepadConnectedEvent {
  event: 'on-connect';
  data: {
    gamepadId: number;
    name: string;
  };
}

export interface GamepadDisconnectedEvent {
  event: 'on-disconnect';
  data: { gamepadId: number };
}

export interface GamepadButtonEvent {
  event: 'on-button';
  data: {
    gamepadId: number;
    button: GamepadButton;
    state: ElementState;
    value: number; // 0.0-1.0 for analog triggers
  };
}

export interface GamepadAxisEvent {
  event: 'on-axis';
  data: {
    gamepadId: number;
    axis: GamepadAxis;
    value: number; // -1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers
  };
}

export type GamepadEvent =
  | GamepadConnectedEvent
  | GamepadDisconnectedEvent
  | GamepadButtonEvent
  | GamepadAxisEvent;
