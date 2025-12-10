// Re-export all command types
export * from './common';
export * from './window';
export * from './render';
export * from './results';
export * from './render-results';
export * from './material-helpers';
export * from './viewport-helpers';
export * from './layer-masks';
export * from './uniform-values';

import type { WindowCmd } from './window';
import type { RenderCmd } from './render';
import type { WindowCmdResult } from './results';
import type { RenderCmdResult } from './render-results';
import type {
  WindowEvent,
  PointerEvent,
  KeyboardEvent,
  GamepadEvent,
  SystemEvent,
} from '../events';

// MARK: Command Union

export type EngineCmd = WindowCmd | RenderCmd;

export type EngineCmdEnvelope = EngineCmd & { id: number };

export type EngineBatchCmds = EngineCmdEnvelope[];

// MARK: Command Responses (responses to commands sent via vulframSendQueue)

export type CommandResponse = WindowCmdResult | RenderCmdResult;

export type CommandResponseEnvelope = CommandResponse & { id: number };

export type EngineBatchResponses = CommandResponseEnvelope[];

// MARK: Engine Events (spontaneous events from input, window changes, system)

export type EngineEvent =
  | { type: 'window'; content: WindowEvent }
  | { type: 'pointer'; content: PointerEvent }
  | { type: 'keyboard'; content: KeyboardEvent }
  | { type: 'gamepad'; content: GamepadEvent }
  | { type: 'system'; content: SystemEvent };

export type EngineBatchEvents = EngineEvent[];
