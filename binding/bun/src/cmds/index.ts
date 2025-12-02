// Re-export all command types
export * from './common';
export * from './window';
export * from './results';

import type { WindowCmd } from './window';
import type { WindowCmdResult } from './results';
import type {
  WindowEvent,
  PointerEvent,
  KeyboardEvent,
  GamepadEvent,
  SystemEvent,
} from '../events';

// MARK: Command Union

export type EngineCmd = WindowCmd;

export interface EngineCmdEnvelope {
  id: number;
  type: EngineCmd['type'];
  content: EngineCmd['content'];
}

export type EngineBatchCmds = EngineCmdEnvelope[];

// MARK: Engine Events (includes both events and command results)

export type EngineEventContent =
  | { type: 'window'; content: WindowEvent }
  | { type: 'pointer'; content: PointerEvent }
  | { type: 'keyboard'; content: KeyboardEvent }
  | { type: 'gamepad'; content: GamepadEvent }
  | { type: 'system'; content: SystemEvent }
  | WindowCmdResult;

export interface EngineEventEnvelope {
  id: number;
  type: EngineEventContent['type'];
  content: EngineEventContent['content'];
}

export type EngineBatchEvents = EngineEventEnvelope[];
