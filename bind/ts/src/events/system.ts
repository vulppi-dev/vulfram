// MARK: System Events

/**
 * Event fired when the application resumes from a suspended state.
 *
 * @example
 * ```typescript
 * const event: SystemResumedEvent = {
 *   event: 'on-resume'
 * };
 * ```
 */
export interface SystemResumedEvent {
  event: 'on-resume';
}

export interface SystemSuspendedEvent {
  event: 'on-suspend';
}

/**
 * Event fired when the system is running low on memory.
 * The application should release non-essential resources.
 *
 * @example
 * ```typescript
 * const event: SystemMemoryWarningEvent = {
 *   event: 'on-memory-warning'
 * };
 * // Free caches, unload unused assets
 * ```
 */
export interface SystemMemoryWarningEvent {
  event: 'on-memory-warning';
}

/**
 * Event fired when the application is about to exit.
 * Final chance to save state and perform cleanup.
 *
 * @example
 * ```typescript
 * const event: SystemExitingEvent = {
 *   event: 'on-exit'
 * };
 * // Save game state, close resources
 * ```
 */
export interface SystemExitingEvent {
  event: 'on-exit';
}

export type SystemEvent =
  | SystemResumedEvent
  | SystemSuspendedEvent
  | SystemMemoryWarningEvent
  | SystemExitingEvent;
