import type { Vector2, ElementState, TouchPhase } from './common';

// MARK: Pointer Events

/** Mouse button types */
export type MouseButton =
  | 'left'
  | 'right'
  | 'middle'
  | 'back'
  | 'forward'
  | { other: number };

/** Pointer type for unified mouse/touch handling */
export type PointerType = 'mouse' | 'touch' | 'pen';

/** Mouse scroll delta type */
export type ScrollDelta =
  | { type: 'line'; value: Vector2 }
  | { type: 'pixel'; value: Vector2 };

export interface PointerMovedEvent {
  event: 'on-move';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
    position: Vector2;
  };
}

export interface PointerEnteredEvent {
  event: 'on-enter';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

export interface PointerLeftEvent {
  event: 'on-leave';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

export interface PointerButtonEvent {
  event: 'on-button';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
    button: MouseButton;
    state: ElementState;
    position: Vector2;
  };
}

export interface PointerScrollEvent {
  event: 'on-scroll';
  data: {
    windowId: number;
    delta: ScrollDelta;
    phase: TouchPhase;
  };
}

export interface PointerTouchEvent {
  event: 'on-touch';
  data: {
    windowId: number;
    pointerId: number;
    phase: TouchPhase;
    position: Vector2;
    pressure: number | null;
  };
}

export interface PointerPinchGestureEvent {
  event: 'on-pinch-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerPanGestureEvent {
  event: 'on-pan-gesture';
  data: {
    windowId: number;
    delta: Vector2;
    phase: TouchPhase;
  };
}

export interface PointerRotationGestureEvent {
  event: 'on-rotation-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerDoubleTapGestureEvent {
  event: 'on-double-tap-gesture';
  data: { windowId: number };
}

export type PointerEvent =
  | PointerMovedEvent
  | PointerEnteredEvent
  | PointerLeftEvent
  | PointerButtonEvent
  | PointerScrollEvent
  | PointerTouchEvent
  | PointerPinchGestureEvent
  | PointerPanGestureEvent
  | PointerRotationGestureEvent
  | PointerDoubleTapGestureEvent;
