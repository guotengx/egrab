# Protocol: Scraper Engine Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-09
- 依赖的真相源：`docs/PRD.md` 1.0.0 第3.1/3.2/3.3节、`docs/ARCHITECTURE.md` 1.0.0 第3.1/4.1/5节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type { JsonObject, ProductData, TaskId, TaskResult } from './data-models';

/** 抓取引擎接口；Rust 模块路径为 src-tauri/src/scraper/。 */
export interface ScraperEngine {
  /** 启动单商品抓取任务。 */
  start_scrape(url: string, force?: boolean): Promise<TaskId>;

  /** 取消任务。 */
  cancel_scrape(task_id: TaskId): Promise<boolean>;

  /** 内部执行完整抓取流程，完成后发出 scrape:complete。 */
  run_task(task_id: TaskId): Promise<TaskResult>;
}

/** 解析阶段内部结果。 */
export interface ScraperParseOutput {
  /** 标准化商品数据。 */
  product: ProductData;
  /** 原始抓取数据；Rust 对应 HashMap<String, serde_json::Value>。 */
  raw_data: JsonObject;
}
```

## 约束
- MVP 同时最多 1 个活动抓取任务；并发调用 `start_scrape` 必须返回可恢复错误 `TASK_ALREADY_RUNNING`。
- 标准流程必须遵守 ARCHITECTURE 4.1：CDP connect/navigate → progress → evaluate/parse → download_images → storage save → complete。
- `force` 语义必须传递给 storage：默认 false，重复任务返回 `DUPLICATE_TASK`；true 时强制重抓。
- 任务取消：`pending`/`running` 可取消；取消后停止 CDP 导航/下载后续工作，释放资源，最终状态为 `cancelled`。
- 事件必须遵守 `docs/protocols/events.md` 的时序和 percent 单调约束。

## 示例

```json
{
  "start_scrape": { "url": "https://item.jd.com/12345678.html", "force": false, "returns": "task_20260509_000001" },
  "complete": { "task_id": "task_20260509_000001", "status": "success", "folder_path": "~/EGrab/jd_12345678_20260509T100000" }
}
```
