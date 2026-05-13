# Protocol: Downloader Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-09
- 依赖的真相源：`docs/PRD.md` 1.0.0 第3.1.3/4.1节、`docs/ARCHITECTURE.md` 1.0.0 第3.1/9节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type { ImageRef, ImageType, ScrapeErrorInfo, TaskId } from './data-models';

/** 图片下载器接口；Rust 模块路径为 src-tauri/src/downloader/。 */
export interface ImageDownloader {
  /** 批量下载图片，遵守并发和重试策略。 */
  download_images(task_id: TaskId, images: DownloadImageInput[], options: DownloadOptions): Promise<DownloadBatchResult>;
}

/** 单张图片下载输入。 */
export interface DownloadImageInput {
  /** 图片类型，对应 cover/gallery/detail/sku 子目录。 */
  type: ImageType;
  /** 图片引用。 */
  image: ImageRef;
  /** 目标相对文件名，例如 cover/cover_001.jpg。 */
  relative_path: string;
}

/** 下载配置。 */
export interface DownloadOptions {
  /** 并发数，默认 3，最大 10，最小 1。 */
  concurrency: number;
  /** 每张图片最大尝试次数，默认 3。 */
  max_attempts: number;
}

/** 单张图片下载结果。 */
export interface DownloadImageResult {
  /** 输入图片类型。 */
  type: ImageType;
  /** 原图 URL。 */
  original_url: string;
  /** 成功时本地相对路径；失败时为 null。 */
  local_path: string | null;
  /** 宽度像素，未知时为 null。 */
  width: number | null;
  /** 高度像素，未知时为 null。 */
  height: number | null;
  /** 文件大小字节数，未知或失败时为 null。 */
  size_bytes: number | null;
  /** 错误；成功时为 null。 */
  error: ScrapeErrorInfo | null;
}

/** 批量下载结果。 */
export interface DownloadBatchResult {
  /** 总图片数。 */
  total: number;
  /** 成功数量。 */
  success: number;
  /** 失败数量。 */
  failed: number;
  /** 每张图片结果。 */
  results: DownloadImageResult[];
}
```

## 约束
- 原图 URL 清洗由 `parser` 负责；`downloader` 优先下载 `ImageRef.original_url`，失败后可降级下载 `thumbnail_url` 并记录 `IMAGE_DOWNLOAD_FAILED`。
- 单张图片失败不得中断整批下载；最终由 scraper 决定任务是否为 `partial`。
- 默认并发数为 3，最大 10；超过范围必须返回 `CONFIG_INVALID` 或自动夹紧并记录警告。
- 每张图片默认总尝试 3 次；重试不得造成无限循环。
- 文件写入路径必须位于任务存档目录下；SKU 文件名和任何由页面文本派生的文件名必须遵守 `storage-interface.md` 中的跨平台文件名安全规则（含 Windows 保留名、非法字符、尾随空格/句点限制）。

## 示例

```json
{
  "total": 2,
  "success": 1,
  "failed": 1,
  "results": [
    { "type": "cover", "original_url": "https://img.example.com/cover.jpg", "local_path": "cover/cover_001.jpg", "width": 800, "height": 800, "size_bytes": 102400, "error": null },
    { "type": "detail", "original_url": "https://img.example.com/detail.jpg", "local_path": null, "width": null, "height": null, "size_bytes": null, "error": { "step": "downloading", "code": "IMAGE_DOWNLOAD_FAILED", "message": "下载失败", "recoverable": true } }
  ]
}
```
