// MARK: System Events

export interface SystemResumedEvent {
  event: 'on-resume';
}

export interface SystemSuspendedEvent {
  event: 'on-suspend';
}

export interface SystemMemoryWarningEvent {
  event: 'on-memory-warning';
}

export interface SystemExitingEvent {
  event: 'on-exit';
}

export type SystemEvent =
  | SystemResumedEvent
  | SystemSuspendedEvent
  | SystemMemoryWarningEvent
  | SystemExitingEvent;
