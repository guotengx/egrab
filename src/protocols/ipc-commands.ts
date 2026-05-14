// EGrab - IPC Commands Protocol (L5)
// Derived from: docs/protocols/ipc-commands.md v1.0.0

import type {
  AppConfig,
  ConnectionInfo,
  ConnectionState,
  IpcError,
  TabInfo,
  TaskDetail,
  TaskFilter,
  TaskId,
  TaskSummary,
} from './data-models';

export interface CdpConnectCommand {
  name: 'cdp_connect';
  params: { port: number };
  returns: ConnectionInfo;
}

export interface CdpDisconnectCommand {
  name: 'cdp_disconnect';
  params: Record<string, never>;
  returns: boolean;
}

export interface CdpStatusCommand {
  name: 'cdp_status';
  params: Record<string, never>;
  returns: ConnectionState;
}

export interface CdpListTabsCommand {
  name: 'cdp_list_tabs';
  params: Record<string, never>;
  returns: TabInfo[];
}

/** 自动检测本地 CDP 端口 → 扫描系统浏览器 → 启动浏览器（带 CDP 参数）→ 连接 CDP。 */
export interface CdpAutoConnectCommand {
  name: 'cdp_auto_connect';
  params: Record<string, never>;
  returns: ConnectionInfo;
}

export interface StartScrapeCommand {
  name: 'start_scrape';
  params: {
    url: string;
    force?: boolean;
  };
  returns: TaskId;
}

export interface CancelScrapeCommand {
  name: 'cancel_scrape';
  params: { task_id: string };
  returns: boolean;
}

export interface GetTaskHistoryCommand {
  name: 'get_task_history';
  params: { filter: TaskFilter };
  returns: TaskSummary[];
}

export interface GetTaskDetailCommand {
  name: 'get_task_detail';
  params: { task_id: string };
  returns: TaskDetail;
}

export interface OpenFolderCommand {
  name: 'open_folder';
  params: { path: string };
  returns: boolean;
}

export interface DeleteTaskCommand {
  name: 'delete_task';
  params: { task_id: string };
  returns: boolean;
}

export interface GetConfigCommand {
  name: 'get_config';
  params: Record<string, never>;
  returns: AppConfig;
}

export interface SetConfigCommand {
  name: 'set_config';
  params: { config: AppConfig };
  returns: boolean;
}

export interface ResizeImagesCommand {
  name: 'resize_images';
  params: { task_id: string };
  returns: {
    total: number;
    resized: number;
    skipped: number;
    failed: number;
    details: Array<{
      path: string;
      original_width: number;
      original_height: number;
      new_width: number | null;
      new_height: number | null;
      action: string;
      error: string | null;
    }>;
  };
  errors: 'UNKNOWN_ERROR';
}

export type IpcResult<T> = { ok: true; data: T } | { ok: false; error: IpcError };

export type IpcCommand =
  | CdpConnectCommand
  | CdpDisconnectCommand
  | CdpStatusCommand
  | CdpListTabsCommand
  | CdpAutoConnectCommand
  | StartScrapeCommand
  | CancelScrapeCommand
  | GetTaskHistoryCommand
  | GetTaskDetailCommand
  | OpenFolderCommand
  | DeleteTaskCommand
  | GetConfigCommand
  | SetConfigCommand
  | ResizeImagesCommand;
