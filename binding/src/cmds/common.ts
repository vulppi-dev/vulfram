import type { IVector2, Size } from '../events';

// MARK: Window State

export type WindowState =
  | 'minimized'
  | 'maximized'
  | 'windowed'
  | 'fullscreen'
  | 'windowed-fullscreen';

// MARK: Cursor

export type CursorGrabMode = 'none' | 'confined' | 'locked';

export type CursorIcon =
  | 'default'
  | 'context-menu'
  | 'help'
  | 'pointer'
  | 'progress'
  | 'wait'
  | 'cell'
  | 'crosshair'
  | 'text'
  | 'vertical-text'
  | 'alias'
  | 'copy'
  | 'move'
  | 'no-drop'
  | 'not-allowed'
  | 'grab'
  | 'grabbing'
  | 'e-resize'
  | 'n-resize'
  | 'ne-resize'
  | 'nw-resize'
  | 's-resize'
  | 'se-resize'
  | 'sw-resize'
  | 'w-resize'
  | 'ew-resize'
  | 'ns-resize'
  | 'nesw-resize'
  | 'nwse-resize'
  | 'col-resize'
  | 'row-resize'
  | 'all-scroll'
  | 'zoom-in'
  | 'zoom-out';

// MARK: Attention

export type UserAttentionType = 'critical' | 'informational';
