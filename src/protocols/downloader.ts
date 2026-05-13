// EGrab - Downloader Interface Protocol (L5)
// Derived from: docs/protocols/downloader-interface.md v1.0.0

import type { ImageRef, ImageType, ScrapeErrorInfo, TaskId } from './data-models';

/** 图片下载器接口；Rust 模块路径为 src-tauri/src/downloader/。 */
export interface ImageDownloader {
  /** 批量下载图片，遵守并发和重试策略。 */
  download_images(
    task_id: TaskId,
    images: DownloadImageInput[],
    options: DownloadOptions,
  ): Promise<DownloadBatchResult>;
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
