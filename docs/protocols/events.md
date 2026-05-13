# Protocol: Events

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-08
- 依赖的真相源：`docs/ARCHITECTURE.md` 1.0.0 第5.2节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type { ConnectionState, ScrapeStep, TaskResult } from './data-models';

/** 抓取进度事件 payload。 */
export interface ScrapeProgressPayload {
  /** 任务 ID。 */
  task_id: string;
  /** 整体百分比，0-100。 */
  percent: number;
  /** 当前步骤。 */
  step: ScrapeStep;
  /** 用户可读进度说明。 */
  message: string;
}

/** 抓取完成事件 payload。 */
export interface ScrapeCompletePayload {
  /** 任务 ID。 */
  task_id: string;
  /** 抓取结果。 */
  result: TaskResult;
}

/** 抓取错误事件 payload。 */
export interface ScrapeErrorPayload {
  /** 任务 ID。 */
  task_id: string;
  /** 用户可读错误信息。 */
  error: string;
  /** 是否可恢复；true 表示流程可继续或可降级。 */
  recoverable: boolean;
}

/** CDP 连接状态变更 payload 直接使用 ConnectionState。 */
export type CdpStateChangedPayload = ConnectionState;

/** 允许的后端到前端事件。 */
export type BackendEvent =
  | { name: 'scrape:progress'; payload: ScrapeProgressPayload }
  | { name: 'scrape:complete'; payload: ScrapeCompletePayload }
  | { name: 'scrape:error'; payload: ScrapeErrorPayload }
  | { name: 'cdp:state_changed'; payload: CdpStateChangedPayload };
```

## 约束
- 事件名只能为：`scrape:progress`, `scrape:complete`, `scrape:error`, `cdp:state_changed`。
- `scrape:progress.percent` 必须在 0 到 100 之间；进度应与核心流程相符：连接、页面加载、数据解析、图片下载、存档写入。
- 事件时序：同一 `task_id` 正常流程应为若干 `scrape:progress` → 一次 `scrape:complete`；可恢复错误可在 progress 之间发送 `scrape:error`；不可恢复错误应发送 `scrape:error` 后再发送一次表示 failed 的 `scrape:complete`，或由命令错误直接失败但不得再发送 success。
- `scrape:progress.percent` 对同一任务必须单调不下降；建议阶段百分比：连接 0-10，页面加载 10-30，数据解析 30-50，图片下载 50-85，存档写入 85-100。
- `scrape:complete.result.task_id` 必须与 payload 顶层 `task_id` 一致。
- `scrape:error.recoverable = true` 表示错误可降级、可重试或不影响整体流程继续；图片单张失败应汇总并允许 partial。`recoverable = false` 表示当前任务无法达成目标，最终状态必须为 `failed` 或 `cancelled`。
- `cdp:state_changed` 必须在连接、断开、重连、失败时发出，payload 与 `ConnectionState` 完全一致。
- 取消任务时，不得在 `cancelled` 终态后继续发送该任务的 `scrape:progress`；允许发送一次 `scrape:complete`，其 `result.status` 为 `cancelled`。

## 示例

```json
{
  "name": "scrape:progress",
  "payload": {
    "task_id": "task_20260508_000001",
    "percent": 40,
    "step": "parsing",
    "message": "正在解析商品数据"
  }
}
```

```json
{
  "name": "cdp:state_changed",
  "payload": {
    "type": "Connected",
    "browser_version": "Chrome/124.0.0.0"
  }
}
```
