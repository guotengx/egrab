// EGrab - Events Protocol (L5)
// Derived from: docs/protocols/events.md v1.0.0

import type { ConnectionState, ScrapeStep, TaskResult } from './data-models';

export interface ScrapeProgressPayload {
  task_id: string;
  percent: number;
  step: ScrapeStep;
  message: string;
}

export interface ScrapeCompletePayload {
  task_id: string;
  result: TaskResult;
}

export interface ScrapeErrorPayload {
  task_id: string;
  error: string;
  recoverable: boolean;
}

export type CdpStateChangedPayload = ConnectionState;

export type BackendEvent =
  | { name: 'scrape:progress'; payload: ScrapeProgressPayload }
  | { name: 'scrape:complete'; payload: ScrapeCompletePayload }
  | { name: 'scrape:error'; payload: ScrapeErrorPayload }
  | { name: 'cdp:state_changed'; payload: CdpStateChangedPayload };
