// EGrab - Event Listener Service
// Encapsulates Tauri event listening with typed callbacks
// Strictly follows src/protocols/ event definitions

import { listen } from '@tauri-apps/api/event';
import type {
  ScrapeProgressPayload,
  ScrapeCompletePayload,
  ScrapeErrorPayload,
  CdpStateChangedPayload,
} from '../protocols';

/**
 * Listen for scrape progress events.
 * Backend event: `scrape:progress`
 * Returns an unlisten function to clean up the listener.
 */
export function onScrapeProgress(
  callback: (payload: ScrapeProgressPayload) => void
): Promise<() => void> {
  return listen<ScrapeProgressPayload>('scrape:progress', (event) => {
    callback(event.payload);
  }).then((unlisten) => () => unlisten());
}

/**
 * Listen for scrape completion events.
 * Backend event: `scrape:complete`
 * Returns an unlisten function to clean up the listener.
 */
export function onScrapeComplete(
  callback: (payload: ScrapeCompletePayload) => void
): Promise<() => void> {
  return listen<ScrapeCompletePayload>('scrape:complete', (event) => {
    callback(event.payload);
  }).then((unlisten) => () => unlisten());
}

/**
 * Listen for scrape error events.
 * Backend event: `scrape:error`
 * Returns an unlisten function to clean up the listener.
 */
export function onScrapeError(
  callback: (payload: ScrapeErrorPayload) => void
): Promise<() => void> {
  return listen<ScrapeErrorPayload>('scrape:error', (event) => {
    callback(event.payload);
  }).then((unlisten) => () => unlisten());
}

/**
 * Listen for CDP state change events.
 * Backend event: `cdp:state_changed`
 * Returns an unlisten function to clean up the listener.
 */
export function onCdpStateChanged(
  callback: (payload: CdpStateChangedPayload) => void
): Promise<() => void> {
  return listen<CdpStateChangedPayload>('cdp:state_changed', (event) => {
    callback(event.payload);
  }).then((unlisten) => () => unlisten());
}
