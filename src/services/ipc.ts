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

// ---------------------------------------------------------------------------
// 抓取规则包 / 页面诊断快照
//
// 平台改版时，规则文件可直接在磁盘上编辑，保存后下一次抓取即生效，
// 无需重新编译或重装程序。
// ---------------------------------------------------------------------------

/** 单个平台的规则摘要。 */
export interface RulePlatformInfo {
  id: string;
  label: string;
  match_url: string[];
  extract_js_file: string | null;
  expand_js_file: string | null;
  has_extract_js: boolean;
}

/** 当前生效的规则包信息。 */
export interface RulesInfo {
  source: 'disk' | 'embedded';
  path: string;
  rules_dir: string;
  snapshots_dir: string;
  version: number;
  updated_at: string;
  error: string | null;
  platforms: RulePlatformInfo[];
}

/**
 * 查询当前生效的抓取规则包。
 * IPC command: `get_rules_info`
 */
export async function getRulesInfo(): Promise<RulesInfo> {
  try {
    return await invoke<RulesInfo>('get_rules_info');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * 校验磁盘上的规则文件是否合法（规则每次抓取都会重新读取，本命令用于提前发现语法错误）。
 * IPC command: `reload_rules`
 */
export async function reloadRules(): Promise<RulesInfo> {
  try {
    return await invoke<RulesInfo>('reload_rules');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * 打开规则目录，供用户直接编辑规则文件。
 * IPC command: `open_rules_folder`
 */
export async function openRulesFolder(): Promise<boolean> {
  try {
    return await invoke<boolean>('open_rules_folder');
  } catch (err) {
    throw toIpcError(err);
  }
}

/**
 * 导出当前浏览器页面的诊断快照（DOM + 候选全局变量 + 图片清单）。
 * 返回生成的 JSON 快照文件路径。
 * IPC command: `dump_page_snapshot`
 */
export async function dumpPageSnapshot(): Promise<string> {
  try {
    return await invoke<string>('dump_page_snapshot');
  } catch (err) {
    throw toIpcError(err);
  }
}
