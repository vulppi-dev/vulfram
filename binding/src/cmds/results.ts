import type { IVector2, Size } from '../events';
import type { WindowState } from './common';

// MARK: Command Results Base

interface CmdResultBase {
  success: boolean;
  message: string;
}

// MARK: Window Lifecycle Results

export interface CmdResultWindowCreate extends CmdResultBase {
  content: number; // window_id
}

export interface CmdResultWindowClose extends CmdResultBase {}

// MARK: Window Property Results

export interface CmdResultWindowSetTitle extends CmdResultBase {}

export interface CmdResultWindowSetPosition extends CmdResultBase {}

export interface CmdResultWindowGetPosition extends CmdResultBase {
  content: IVector2;
}

export interface CmdResultWindowSetSize extends CmdResultBase {}

export interface CmdResultWindowGetSize extends CmdResultBase {
  content: Size;
}

export interface CmdResultWindowGetOuterSize extends CmdResultBase {
  content: Size;
}

export interface CmdResultWindowGetSurfaceSize extends CmdResultBase {
  content: Size;
}

// MARK: Window State Results

export interface CmdResultWindowSetState extends CmdResultBase {}

export interface CmdResultWindowGetState extends CmdResultBase {
  content: WindowState;
}

export interface CmdResultWindowSetIcon extends CmdResultBase {}

// MARK: Window Decoration Results

export interface CmdResultWindowSetDecorations extends CmdResultBase {}

export interface CmdResultWindowHasDecorations extends CmdResultBase {
  content: boolean;
}

export interface CmdResultWindowSetResizable extends CmdResultBase {}

export interface CmdResultWindowIsResizable extends CmdResultBase {
  content: boolean;
}

// MARK: Window Focus Results

export interface CmdResultWindowRequestAttention extends CmdResultBase {}

export interface CmdResultWindowFocus extends CmdResultBase {}

// MARK: Window Cursor Results

export interface CmdResultWindowSetCursorVisible extends CmdResultBase {}

export interface CmdResultWindowSetCursorGrab extends CmdResultBase {}

export interface CmdResultWindowSetCursorIcon extends CmdResultBase {}

// MARK: Result Union

export type WindowCmdResult =
  | { type: 'window-create'; content: CmdResultWindowCreate }
  | { type: 'window-close'; content: CmdResultWindowClose }
  | { type: 'window-set-title'; content: CmdResultWindowSetTitle }
  | { type: 'window-set-position'; content: CmdResultWindowSetPosition }
  | { type: 'window-get-position'; content: CmdResultWindowGetPosition }
  | { type: 'window-set-size'; content: CmdResultWindowSetSize }
  | { type: 'window-get-size'; content: CmdResultWindowGetSize }
  | { type: 'window-get-outer-size'; content: CmdResultWindowGetOuterSize }
  | { type: 'window-get-surface-size'; content: CmdResultWindowGetSurfaceSize }
  | { type: 'window-set-state'; content: CmdResultWindowSetState }
  | { type: 'window-get-state'; content: CmdResultWindowGetState }
  | { type: 'window-set-icon'; content: CmdResultWindowSetIcon }
  | { type: 'window-set-decorations'; content: CmdResultWindowSetDecorations }
  | { type: 'window-has-decorations'; content: CmdResultWindowHasDecorations }
  | { type: 'window-set-resizable'; content: CmdResultWindowSetResizable }
  | { type: 'window-is-resizable'; content: CmdResultWindowIsResizable }
  | {
      type: 'window-request-attention';
      content: CmdResultWindowRequestAttention;
    }
  | { type: 'window-focus'; content: CmdResultWindowFocus }
  | {
      type: 'window-set-cursor-visible';
      content: CmdResultWindowSetCursorVisible;
    }
  | { type: 'window-set-cursor-grab'; content: CmdResultWindowSetCursorGrab }
  | { type: 'window-set-cursor-icon'; content: CmdResultWindowSetCursorIcon };
