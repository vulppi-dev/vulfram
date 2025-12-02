import type { IVector2 } from './common';

// MARK: Window Events

export interface WindowCreatedEvent {
  event: 'on-create';
  data: { windowId: number };
}

export interface WindowResizedEvent {
  event: 'on-resize';
  data: { windowId: number; width: number; height: number };
}

export interface WindowMovedEvent {
  event: 'on-move';
  data: { windowId: number; position: IVector2 };
}

export interface WindowCloseRequestedEvent {
  event: 'on-close-request';
  data: { windowId: number };
}

export interface WindowDestroyedEvent {
  event: 'on-destroy';
  data: { windowId: number };
}

export interface WindowFocusedEvent {
  event: 'on-focus';
  data: { windowId: number; focused: boolean };
}

export interface WindowScaleFactorChangedEvent {
  event: 'on-scale-factor-change';
  data: {
    windowId: number;
    scaleFactor: number;
    newWidth: number;
    newHeight: number;
  };
}

export interface WindowOccludedEvent {
  event: 'on-occlude';
  data: { windowId: number; occluded: boolean };
}

export interface WindowRedrawRequestedEvent {
  event: 'on-redraw-request';
  data: { windowId: number };
}

export interface WindowFileDroppedEvent {
  event: 'on-file-drop';
  data: { windowId: number; path: string };
}

export interface WindowFileHoveredEvent {
  event: 'on-file-hover';
  data: { windowId: number; path: string };
}

export interface WindowFileHoveredCancelledEvent {
  event: 'on-file-hover-cancel';
  data: { windowId: number };
}

export interface WindowThemeChangedEvent {
  event: 'on-theme-change';
  data: { windowId: number; darkMode: boolean };
}

export type WindowEvent =
  | WindowCreatedEvent
  | WindowResizedEvent
  | WindowMovedEvent
  | WindowCloseRequestedEvent
  | WindowDestroyedEvent
  | WindowFocusedEvent
  | WindowScaleFactorChangedEvent
  | WindowOccludedEvent
  | WindowRedrawRequestedEvent
  | WindowFileDroppedEvent
  | WindowFileHoveredEvent
  | WindowFileHoveredCancelledEvent
  | WindowThemeChangedEvent;
