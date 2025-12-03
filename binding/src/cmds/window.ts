import type { IVector2, Size } from '../events';
import type {
  WindowState,
  CursorGrabMode,
  CursorIcon,
  UserAttentionType,
} from './common';

// MARK: Window Lifecycle Commands

/**
 * Command to create a new window.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowCreate = {
 *   type: 'cmd-window-create',
 *   content: {
 *     title: 'My Game Window',
 *     size: { width: 1920, height: 1080 },
 *     position: { x: 0, y: 0 },
 *     borderless: false,
 *     resizable: true,
 *     initialState: 'maximized'
 *   }
 * };
 * ```
 */
export interface CmdWindowCreate {
  type: 'cmd-window-create';
  content: {
    title: string;
    size: Size;
    position: IVector2;
    borderless: boolean;
    resizable: boolean;
    initialState: WindowState;
  };
}

/**
 * Command to close an existing window.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowClose = {
 *   type: 'cmd-window-close',
 *   content: { windowId: 1 }
 * };
 * ```
 */
export interface CmdWindowClose {
  type: 'cmd-window-close';
  content: {
    windowId: number;
  };
}

// MARK: Window Property Commands

/**
 * Command to change the window title.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowSetTitle = {
 *   type: 'cmd-window-set-title',
 *   content: {
 *     windowId: 1,
 *     title: 'New Window Title'
 *   }
 * };
 * ```
 */
export interface CmdWindowSetTitle {
  type: 'cmd-window-set-title';
  content: {
    windowId: number;
    title: string;
  };
}

/**
 * Command to set the window position on screen.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowSetPosition = {
 *   type: 'cmd-window-set-position',
 *   content: {
 *     windowId: 1,
 *     position: { x: 100, y: 100 }
 *   }
 * };
 * ```
 */
export interface CmdWindowSetPosition {
  type: 'cmd-window-set-position';
  content: {
    windowId: number;
    position: IVector2;
  };
}

/**
 * Command to retrieve the current window position.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowGetPosition = {
 *   type: 'cmd-window-get-position',
 *   content: { windowId: 1 }
 * };
 * ```
 */
export interface CmdWindowGetPosition {
  type: 'cmd-window-get-position';
  content: {
    windowId: number;
  };
}

/**
 * Command to change the window size (inner client area).
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowSetSize = {
 *   type: 'cmd-window-set-size',
 *   content: {
 *     windowId: 1,
 *     size: { width: 1280, height: 720 }
 *   }
 * };
 * ```
 */
export interface CmdWindowSetSize {
  type: 'cmd-window-set-size';
  content: {
    windowId: number;
    size: Size;
  };
}

export interface CmdWindowGetSize {
  type: 'cmd-window-get-size';
  content: {
    windowId: number;
  };
}

export interface CmdWindowGetOuterSize {
  type: 'cmd-window-get-outer-size';
  content: {
    windowId: number;
  };
}

export interface CmdWindowGetSurfaceSize {
  type: 'cmd-window-get-surface-size';
  content: {
    windowId: number;
  };
}

// MARK: Window State Commands

export interface CmdWindowSetState {
  type: 'cmd-window-set-state';
  content: {
    windowId: number;
    state: WindowState;
  };
}

export interface CmdWindowGetState {
  type: 'cmd-window-get-state';
  content: {
    windowId: number;
  };
}

export interface CmdWindowSetIcon {
  type: 'cmd-window-set-icon';
  content: {
    windowId: number;
    bufferId: number;
  };
}

// MARK: Window Decoration Commands

export interface CmdWindowSetDecorations {
  type: 'cmd-window-set-decorations';
  content: {
    windowId: number;
    decorations: boolean;
  };
}

export interface CmdWindowHasDecorations {
  type: 'cmd-window-has-decorations';
  content: {
    windowId: number;
  };
}

export interface CmdWindowSetResizable {
  type: 'cmd-window-set-resizable';
  content: {
    windowId: number;
    resizable: boolean;
  };
}

export interface CmdWindowIsResizable {
  type: 'cmd-window-is-resizable';
  content: {
    windowId: number;
  };
}

// MARK: Window Focus Commands

export interface CmdWindowRequestAttention {
  type: 'cmd-window-request-attention';
  content: {
    windowId: number;
    attentionType: UserAttentionType | null;
  };
}

export interface CmdWindowFocus {
  type: 'cmd-window-focus';
  content: {
    windowId: number;
  };
}

// MARK: Window Cursor Commands

export interface CmdWindowSetCursorVisible {
  type: 'cmd-window-set-cursor-visible';
  content: {
    windowId: number;
    visible: boolean;
  };
}

export interface CmdWindowSetCursorGrab {
  type: 'cmd-window-set-cursor-grab';
  content: {
    windowId: number;
    mode: CursorGrabMode;
  };
}

/**
 * Command to change the cursor icon appearance.
 *
 * @example
 * ```typescript
 * const cmd: CmdWindowSetCursorIcon = {
 *   type: 'cmd-window-set-cursor-icon',
 *   content: {
 *     windowId: 1,
 *     icon: 'pointer'
 *   }
 * };
 * ```
 */
export interface CmdWindowSetCursorIcon {
  type: 'cmd-window-set-cursor-icon';
  content: {
    windowId: number;
    icon: CursorIcon;
  };
}

// MARK: Command Union

export type WindowCmd =
  | CmdWindowCreate
  | CmdWindowClose
  | CmdWindowSetTitle
  | CmdWindowSetPosition
  | CmdWindowGetPosition
  | CmdWindowSetSize
  | CmdWindowGetSize
  | CmdWindowGetOuterSize
  | CmdWindowGetSurfaceSize
  | CmdWindowSetState
  | CmdWindowGetState
  | CmdWindowSetIcon
  | CmdWindowSetDecorations
  | CmdWindowHasDecorations
  | CmdWindowSetResizable
  | CmdWindowIsResizable
  | CmdWindowRequestAttention
  | CmdWindowFocus
  | CmdWindowSetCursorVisible
  | CmdWindowSetCursorGrab
  | CmdWindowSetCursorIcon;
