# Protocol: IPC Commands

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-08
- 依赖的真相源：`docs/PRD.md` 1.0.0、`docs/ARCHITECTURE.md` 1.0.0 第5.1节、`docs/protocols/data-models.md`

## 类型定义

IPC 命令名称、参数与返回值必须与 ARCHITECTURE 第5.1节完全对齐；其中 `cdp_auto_connect` 为人类确认新增的 L4 协议命令。当前命令总数：12。

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `cdp_connect` | `port: number` | `ConnectionInfo` | 连接指定本地 CDP 端口 |
| `cdp_auto_connect` | 无参数 | `ConnectionInfo` | 自动检测 CDP 端口 → 扫描系统浏览器 → 启动浏览器（带 CDP 参数）→ 连接 CDP |
| `cdp_disconnect` | 无参数 | `boolean` | 断开 CDP 连接 |
| `cdp_status` | 无参数 | `ConnectionState` | 查询 CDP 连接状态 |
| `cdp_list_tabs` | 无参数 | `TabInfo[]` | 列出浏览器标签页 |
| `start_scrape` | `url: string, force?: boolean` | `TaskId` | 开始单商品抓取任务 |
| `cancel_scrape` | `task_id: string` | `boolean` | 取消抓取任务 |
| `get_task_history` | `filter: TaskFilter` | `TaskSummary[]` | 查询抓取任务历史 |
| `get_task_detail` | `task_id: string` | `TaskDetail` | 获取任务详情 |
| `open_folder` | `path: string` | `boolean` | 打开本地存档目录 |
| `get_config` | 无参数 | `AppConfig` | 获取应用配置 |
| `set_config` | `config: AppConfig` | `boolean` | 保存应用配置 |

TypeScript 签名如下：

```ts
import type {
  AppConfig,
  ConnectionInfo,
  ConnectionState,
  IpcError,
  TabInfo,
  TaskDetail,
  TaskFilter,
  TaskId,
  TaskSummary
} from './data-models';

/** 连接 CDP：参数 port 为本地调试端口，默认场景为 9222。 */
export interface CdpConnectCommand {
  name: 'cdp_connect';
  params: { port: number };
  returns: ConnectionInfo;
}

/** 自动检测 CDP 端口、扫描系统浏览器、必要时启动浏览器并连接 CDP。 */
export interface CdpAutoConnectCommand {
  name: 'cdp_auto_connect';
  params: Record<string, never>;
  returns: ConnectionInfo;
}

/** 断开 CDP 连接。 */
export interface CdpDisconnectCommand {
  name: 'cdp_disconnect';
  params: Record<string, never>;
  returns: boolean;
}

/** 查询 CDP 连接状态。 */
export interface CdpStatusCommand {
  name: 'cdp_status';
  params: Record<string, never>;
  returns: ConnectionState;
}

/** 列出浏览器标签页。 */
export interface CdpListTabsCommand {
  name: 'cdp_list_tabs';
  params: Record<string, never>;
  returns: TabInfo[];
}

/** 开始单商品抓取任务。 */
export interface StartScrapeCommand {
  name: 'start_scrape';
  params: {
    /** 商品 URL。 */
    url: string;
    /** 是否强制覆盖同平台同 item_id 的去重限制；默认 false。 */
    force?: boolean;
  };
  returns: TaskId;
}

/** 取消抓取任务。 */
export interface CancelScrapeCommand {
  name: 'cancel_scrape';
  params: { task_id: string };
  returns: boolean;
}

/** 查询抓取任务历史。 */
export interface GetTaskHistoryCommand {
  name: 'get_task_history';
  params: { filter: TaskFilter };
  returns: TaskSummary[];
}

/** 获取任务详情。 */
export interface GetTaskDetailCommand {
  name: 'get_task_detail';
  params: { task_id: string };
  returns: TaskDetail;
}

/** 打开本地存档目录。 */
export interface OpenFolderCommand {
  name: 'open_folder';
  params: { path: string };
  returns: boolean;
}

/** IPC 调用成功/失败的通用语义；Tauri 实现可使用 Result<T, IpcError>。 */
export type IpcResult<T> = { ok: true; data: T } | { ok: false; error: IpcError };

/** 获取应用配置。 */
export interface GetConfigCommand {
  name: 'get_config';
  params: Record<string, never>;
  returns: AppConfig;
}

/** 保存应用配置。 */
export interface SetConfigCommand {
  name: 'set_config';
  params: { config: AppConfig };
  returns: boolean;
}

/** 全部允许的 IPC 命令联合类型。 */
export type IpcCommand =
  | CdpConnectCommand
  | CdpAutoConnectCommand
  | CdpDisconnectCommand
  | CdpStatusCommand
  | CdpListTabsCommand
  | StartScrapeCommand
  | CancelScrapeCommand
  | GetTaskHistoryCommand
  | GetTaskDetailCommand
  | OpenFolderCommand
  | GetConfigCommand
  | SetConfigCommand;
```

## 约束
- 命令名只能为：`cdp_connect`, `cdp_auto_connect`, `cdp_disconnect`, `cdp_status`, `cdp_list_tabs`, `start_scrape`, `cancel_scrape`, `get_task_history`, `get_task_detail`, `open_folder`, `get_config`, `set_config`。
- `cdp_connect.port` 为 `u16` 语义范围：1-65535；实现必须仅连接 `127.0.0.1:{port}`；连接超时 10s。
- `cdp_connect` 只接受显式端口；自动扫描本地 CDP 端口、扫描系统浏览器、启动浏览器（带 CDP 参数）并连接 CDP 的流程必须通过 `cdp_auto_connect` 暴露给前端。`cdp_auto_connect` 不接受参数，默认优先尝试配置的 `cdp_port`（默认 9222），后续端口范围由 `config` 模块或实现策略决定。
- `cdp_auto_connect` 失败时必须返回 `IpcError`；未检测到支持 CDP 的浏览器返回 `NO_BROWSER_FOUND`，浏览器启动后等待 CDP 超时返回 `CDP_LAUNCH_TIMEOUT`，连接 CDP 失败返回 `CDP_CONNECT_FAILED`。
- `start_scrape.url` 必须为淘宝/天猫或京东商品 URL；客户端需验证 URL 合法性并识别平台。MVP URL 规则：淘宝 `^https://item\.taobao\.com/item\.htm\?.*\bid=\d+`，天猫 `^https://detail\.tmall\.com/item\.htm\?.*\bid=\d+`，京东 `^https://item\.jd\.com/\d+\.html`；实现可兼容 `http` 后规范化为 `https`，但不得接受非目标域名。
- `start_scrape.force` 默认 `false`；当同一 `(platform, item_id)` 已存在且 `force=false` 时必须返回 `DUPLICATE_TASK`；`force=true` 时按 storage 协议执行强制重抓。
- `cancel_scrape.task_id`、`get_task_detail.task_id` 必须为已知任务 ID；未知任务应返回 `IpcError`，`code='TASK_NOT_FOUND'`。
- `cancel_scrape` 状态转换：`pending`/`running` 可转为 `cancelled`；`success`/`failed`/`partial`/`cancelled` 为终态，不得重新取消为其他状态；取消后必须停止后续进度事件，允许发送一次 `scrape:complete` 表示 cancelled 结果。
- `get_task_history.filter` 使用 `TaskFilter`；必须支持按平台、时间范围、关键词、状态过滤，并支持 `item_id` 精确查询。
- `open_folder.path` 必须为本地存档路径；不得打开远程 URL。后端必须 canonicalize 并校验路径位于配置的 `storage_root` 或已知任务 `folder_path` 内；非法路径返回 `PATH_NOT_ALLOWED`。
- `set_config.config.image_concurrency` 范围 1-10，默认 3；`cdp_port` 默认 9222。
- 所有命令错误必须序列化为 `IpcError` 给前端；不得 panic 或返回不可解析对象。Rust 端推荐 `Result<T, IpcError>`，前端服务层可统一转换为 `IpcResult<T>`。
- 并发抓取：MVP 同时只允许一个 `start_scrape` 活动任务；已有 `running` 任务时再次调用应返回可恢复错误（建议 code=`TASK_ALREADY_RUNNING`，或归类为 `UNKNOWN_ERROR` 并说明）。

## 示例

```json
{
  "command": "cdp_auto_connect",
  "params": {},
  "returns": {
    "port": 9222,
    "endpoint": "ws://127.0.0.1:9222/devtools/browser/example",
    "browser_version": "Chrome/124.0.0.0",
    "state": {
      "type": "Connected",
      "browser_version": "Chrome/124.0.0.0"
    }
  }
}
```

```json
{
  "command": "start_scrape",
  "params": {
    "url": "https://item.taobao.com/item.htm?id=12345678",
    "force": false
  },
  "returns": "task_20260508_000001"
}
```

```json
{
  "command": "get_task_history",
  "params": {
    "filter": {
      "platform": "taobao",
      "status": "success",
      "keyword": "连衣裙",
      "start_time": "2026-05-01T00:00:00Z",
      "end_time": "2026-05-08T23:59:59Z",
      "limit": 20,
      "offset": 0
    }
  },
  "returns": [
    {
      "id": "task_20260508_000001",
      "url": "https://item.taobao.com/item.htm?id=12345678",
      "platform": "taobao",
      "item_id": "12345678",
      "title": "示例商品",
      "status": "success",
      "created_at": "2026-05-08T10:00:00Z",
      "folder_path": "~/EGrab/taobao_12345678_20260508T100000",
      "cover_path": "~/EGrab/taobao_12345678_20260508T100000/cover/cover_001.jpg"
    }
  ]
}
```
