// EGrab - Scraper Engine Interface Protocol (L5)
// Derived from: docs/protocols/scraper-engine-interface.md v1.0.0

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
