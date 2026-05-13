// EGrab - Connection Store
// Manages CDP connection state using Svelte 5 runes
// Provides connect/disconnect/status methods and listens for cdp:state_changed events

import type { ConnectionState, TabInfo } from '../protocols';
import { cdpAutoConnect, cdpConnect, cdpDisconnect, cdpStatus, cdpListTabs } from '../services/ipc';
import { onCdpStateChanged } from '../services/events';

/** Extract human-readable error message from Tauri IpcError or standard Error. */
function extractErrorMessage(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    return String((err as { message: unknown }).message);
  }
  if (err instanceof Error) return err.message;
  return String(err);
}

/**
 * Reactive connection state store using Svelte 5 runes.
 * Tracks CDP connection state, browser tabs, and auto-updates via events.
 */
function createConnectionStore() {
  // --- Core State ---
  let state = $state<ConnectionState>({ type: 'Disconnected' });
  let tabs = $state<TabInfo[]>([]);
  let isOperating = $state<boolean>(false);
  let error = $state<string | null>(null);

  // --- Event Listener Cleanup ---
  // Retained for future cleanup; assigned in setupEventListener()
  let unlistenStateChanged: (() => void) | null = null;

  // --- Register Event Listener (once on creation) ---
  async function setupEventListener(): Promise<void> {
    unlistenStateChanged = await onCdpStateChanged((payload) => {
      state = payload;
    });
    // Suppress unused variable warning - cleanup will be used on component destroy
    void unlistenStateChanged;
  }

  // Initialize listener immediately
  setupEventListener();

  /** Connect to CDP endpoint on the given port. Updates state on success/failure. */
  async function connect(port: number): Promise<void> {
    isOperating = true;
    error = null;

    try {
      state = { type: 'Connecting' };
      const info = await cdpConnect(port);
      state = info.state;
      // Load tabs after successful connection
      if (info.state.type === 'Connected') {
        await loadTabs();
      }
    } catch (err) {
      const message = extractErrorMessage(err);
      state = { type: 'Failed', reason: message };
      error = `CDP 连接失败: ${message}`;
    } finally {
      isOperating = false;
    }
  }

  /** Auto-connect: detect CDP → detect browser → start → connect. */
  async function autoConnect(): Promise<void> {
    isOperating = true;
    error = null;
    state = { type: 'Connecting' };

    try {
      const info = await cdpAutoConnect();
      state = info.state;
      if (info.state.type === 'Connected') {
        await loadTabs();
      }
    } catch (err) {
      const message = extractErrorMessage(err);
      state = { type: 'Failed', reason: message };
      error = `自动连接失败: ${message}`;
    } finally {
      isOperating = false;
    }
  }

  /** Disconnect from CDP endpoint. Updates state on success/failure. */
  async function disconnect(): Promise<void> {
    isOperating = true;
    error = null;

    try {
      state = { type: 'Disconnected' };
      tabs = [];
      await cdpDisconnect();
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `CDP 断开失败: ${message}`;
    } finally {
      isOperating = false;
    }
  }

  /** Query current CDP connection state from backend. */
  async function checkStatus(): Promise<void> {
    try {
      state = await cdpStatus();
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `查询连接状态失败: ${message}`;
    }
  }

  /** Load browser tabs list from backend. */
  async function loadTabs(): Promise<void> {
    try {
      tabs = await cdpListTabs();
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `获取标签列表失败: ${message}`;
      tabs = [];
    }
  }

  return {
    get state(): ConnectionState {
      return state;
    },
    get tabs(): TabInfo[] {
      return tabs;
    },
    get isOperating(): boolean {
      return isOperating;
    },
    get error(): string | null {
      return error;
    },
    connect,
    autoConnect,
    disconnect,
    checkStatus,
    loadTabs,
  };
}

export const connectionStore = createConnectionStore();
