import type { ElementState, ModifiersState } from './common';

// MARK: Keyboard Events

/** Key location on keyboard */
export type KeyLocation = 'standard' | 'left' | 'right' | 'numpad';

/** Physical key code (scancode-like, layout independent) */
export type KeyCode =
  // Writing System Keys
  | 'backquote'
  | 'backslash'
  | 'bracket-left'
  | 'bracket-right'
  | 'comma'
  | 'digit0'
  | 'digit1'
  | 'digit2'
  | 'digit3'
  | 'digit4'
  | 'digit5'
  | 'digit6'
  | 'digit7'
  | 'digit8'
  | 'digit9'
  | 'equal'
  | 'intl-backslash'
  | 'intl-ro'
  | 'intl-yen'
  | 'key-a'
  | 'key-b'
  | 'key-c'
  | 'key-d'
  | 'key-e'
  | 'key-f'
  | 'key-g'
  | 'key-h'
  | 'key-i'
  | 'key-j'
  | 'key-k'
  | 'key-l'
  | 'key-m'
  | 'key-n'
  | 'key-o'
  | 'key-p'
  | 'key-q'
  | 'key-r'
  | 'key-s'
  | 'key-t'
  | 'key-u'
  | 'key-v'
  | 'key-w'
  | 'key-x'
  | 'key-y'
  | 'key-z'
  | 'minus'
  | 'period'
  | 'quote'
  | 'semicolon'
  | 'slash'
  // Functional Keys
  | 'alt-left'
  | 'alt-right'
  | 'backspace'
  | 'caps-lock'
  | 'context-menu'
  | 'control-left'
  | 'control-right'
  | 'enter'
  | 'super-left'
  | 'super-right'
  | 'shift-left'
  | 'shift-right'
  | 'space'
  | 'tab'
  // Control Keys
  | 'delete'
  | 'end'
  | 'help'
  | 'home'
  | 'insert'
  | 'page-down'
  | 'page-up'
  // Arrow Keys
  | 'arrow-down'
  | 'arrow-left'
  | 'arrow-right'
  | 'arrow-up'
  // Numpad Keys
  | 'num-lock'
  | 'numpad0'
  | 'numpad1'
  | 'numpad2'
  | 'numpad3'
  | 'numpad4'
  | 'numpad5'
  | 'numpad6'
  | 'numpad7'
  | 'numpad8'
  | 'numpad9'
  | 'numpad-add'
  | 'numpad-backspace'
  | 'numpad-clear'
  | 'numpad-clear-entry'
  | 'numpad-comma'
  | 'numpad-decimal'
  | 'numpad-divide'
  | 'numpad-enter'
  | 'numpad-equal'
  | 'numpad-hash'
  | 'numpad-memory-add'
  | 'numpad-memory-clear'
  | 'numpad-memory-recall'
  | 'numpad-memory-store'
  | 'numpad-memory-subtract'
  | 'numpad-multiply'
  | 'numpad-paren-left'
  | 'numpad-paren-right'
  | 'numpad-star'
  | 'numpad-subtract'
  // Function Keys
  | 'escape'
  | 'f1'
  | 'f2'
  | 'f3'
  | 'f4'
  | 'f5'
  | 'f6'
  | 'f7'
  | 'f8'
  | 'f9'
  | 'f10'
  | 'f11'
  | 'f12'
  | 'f13'
  | 'f14'
  | 'f15'
  | 'f16'
  | 'f17'
  | 'f18'
  | 'f19'
  | 'f20'
  | 'f21'
  | 'f22'
  | 'f23'
  | 'f24'
  // Lock Keys
  | 'scroll-lock'
  // Media Keys
  | 'audio-volume-down'
  | 'audio-volume-mute'
  | 'audio-volume-up'
  | 'media-play-pause'
  | 'media-stop'
  | 'media-track-next'
  | 'media-track-previous'
  // Browser Keys
  | 'browser-back'
  | 'browser-favorites'
  | 'browser-forward'
  | 'browser-home'
  | 'browser-refresh'
  | 'browser-search'
  | 'browser-stop'
  // System Keys
  | 'print-screen'
  | 'pause'
  // Unknown
  | 'unidentified';

/**
 * Event fired when a keyboard key is pressed or released.
 *
 * @example
 * ```typescript
 * const event: KeyboardInputEvent = {
 *   event: 'on-input',
 *   data: {
 *     windowId: 1,
 *     keyCode: 'key-w',
 *     state: 'pressed',
 *     location: 'standard',
 *     repeat: false,
 *     text: 'w',
 *     modifiers: { shift: false, ctrl: false, alt: false, meta: false }
 *   }
 * };
 * ```
 */
export interface KeyboardInputEvent {
  event: 'on-input';
  data: {
    windowId: number;
    keyCode: KeyCode;
    state: ElementState;
    location: KeyLocation;
    repeat: boolean;
    text: string | null;
    modifiers: ModifiersState;
  };
}

export interface KeyboardModifiersChangedEvent {
  event: 'on-modifiers-change';
  data: {
    windowId: number;
    modifiers: ModifiersState;
  };
}

export interface KeyboardImeEnabledEvent {
  event: 'on-ime-enable';
  data: { windowId: number };
}

export interface KeyboardImePreeditEvent {
  event: 'on-ime-preedit';
  data: {
    windowId: number;
    text: string;
    cursorRange: [number, number] | null;
  };
}

export interface KeyboardImeCommitEvent {
  event: 'on-ime-commit';
  data: {
    windowId: number;
    text: string;
  };
}

export interface KeyboardImeDisabledEvent {
  event: 'on-ime-disable';
  data: { windowId: number };
}

export type KeyboardEvent =
  | KeyboardInputEvent
  | KeyboardModifiersChangedEvent
  | KeyboardImeEnabledEvent
  | KeyboardImePreeditEvent
  | KeyboardImeCommitEvent
  | KeyboardImeDisabledEvent;
