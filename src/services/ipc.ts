// EGrab - IPC Service Layer
// Encapsulates all Tauri invoke calls with type-safe interfaces
// Strictly follows src/protocols/ command definitions

import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  ConnectionInfo,
  ConnectionState,
  ResizeResult,
  TabInfo,
  TaskDetail,
  TaskFilter,
  TaskId,
  TaskSummary,
  IpcError,
} from '../protocols';

/**
 * Converts an unknown error from Tauri invoke into a standardized IpcError.
 */
function toIpcError(err: unknown): IpcError {
  if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
    return err as IpcError;
  }
  const message = err instanceof Error ? err.message : String(err);
  return {
    code: 'UNKNOWN_ERROR',
    message,
    recoverable: false,
    step: null,
  };
}

/**
 * Get application configuration from backend.
 * IPC command: `get_config`
 */
export async function getConfig(): Promise<AppConfig> {
  try {
    return await invoke<AppConfig>('get_config');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Save application configuration to backend.
 * IPC command: `set_config`
 */
export async function setConfig(config: AppConfig): Promise<boolean> {
  try {
    return await invoke<boolean>('set_config', { config });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Auto-detect and connect to CDP (backend handles browser detection/startup/connect).
 * IPC command: `cdp_auto_connect`
 */
export async function cdpAutoConnect(): Promise<ConnectionInfo> {
  try {
    return await invoke<ConnectionInfo>('cdp_auto_connect');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Connect to CDP endpoint.
 * IPC command: `cdp_connect`
 * TODO: Full implementation in next batch
 */
export async function cdpConnect(port: number): Promise<ConnectionInfo> {
  try {
    return await invoke<ConnectionInfo>('cdp_connect', { port });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Disconnect from CDP.
 * IPC command: `cdp_disconnect`
 * TODO: Full implementation in next batch
 */
export async function cdpDisconnect(): Promise<boolean> {
  try {
    return await invoke<boolean>('cdp_disconnect');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Query current CDP connection state.
 * IPC command: `cdp_status`
 * TODO: Full implementation in next batch
 */
export async function cdpStatus(): Promise<ConnectionState> {
  try {
    return await invoke<ConnectionState>('cdp_status');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * List all browser tabs via CDP.
 * IPC command: `cdp_list_tabs`
 * TODO: Full implementation in next batch
 */
export async function cdpListTabs(): Promise<TabInfo[]> {
  try {
    return await invoke<TabInfo[]>('cdp_list_tabs');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Start a scrape task for the given URL.
 * IPC command: `start_scrape`
 * When `force` is not provided, it defaults to `false`.
 * TODO: Full implementation in next batch
 */
export async function startScrape(url: string, force: boolean = false): Promise<TaskId> {
  try {
    return await invoke<TaskId>('start_scrape', { url, force });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Cancel a running scrape task.
 * IPC command: `cancel_scrape`
 * TODO: Full implementation in next batch
 */
export async function cancelScrape(taskId: string): Promise<boolean> {
  try {
    return await invoke<boolean>('cancel_scrape', { taskId });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Query task history with optional filters.
 * IPC command: `get_task_history`
 * TODO: Full implementation in next batch
 */
export async function getTaskHistory(filter: TaskFilter): Promise<TaskSummary[]> {
  try {
    return await invoke<TaskSummary[]>('get_task_history', { filter });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Get full detail of a specific task.
 * IPC command: `get_task_detail`
 * TODO: Full implementation in next batch
 */
export async function getTaskDetail(taskId: string): Promise<TaskDetail> {
  try {
    return await invoke<TaskDetail>('get_task_detail', { taskId });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Navigate the CDP-connected browser to a specified URL.
 * IPC command: `cdp_navigate`
 */
export async function cdpNavigate(url: string): Promise<void> {
  try {
    await invoke<void>('cdp_navigate', { url });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Open a local folder in system file explorer.
 * IPC command: `open_folder`
 * TODO: Full implementation in next batch
 */
export async function openFolder(path: string): Promise<boolean> {
  try {
    return await invoke<boolean>('open_folder', { path });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Delete a task and its associated data.
 * IPC command: `delete_task`
 */
export async function deleteTask(taskId: string): Promise<boolean> {
  try {
    return await invoke<boolean>('delete_task', { taskId });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Get cover image of a task as base64 data URL.
 * IPC command: `get_cover_image`
 */
export async function getCoverImage(taskId: string): Promise<string> {
  try {
    return await invoke<string>('get_cover_image', { taskId });
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * Resize oversized images in a task's local folder.
 * IPC command: `resize_images`
 */
export async function resizeImages(taskId: string): Promise<ResizeResult> {
  try {
    return await invoke<ResizeResult>('resize_images', { taskId });
  } catch (err) {
    throw toIpcError(err);
  }
}
