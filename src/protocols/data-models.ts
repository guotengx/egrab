// EGrab - Data Models Protocol (L5)
// Derived from: docs/protocols/data-models.md v1.0.0
// Field names must match PRD 3.1.2 exactly

export type KnownPlatform = 'taobao' | 'tmall' | 'jd';

export type Platform = KnownPlatform | (string & { readonly __platform_extension?: never });

export type TaskStatus = 'pending' | 'running' | 'success' | 'failed' | 'partial' | 'cancelled';

export type ImageType = 'cover' | 'gallery' | 'detail' | 'sku';

export type ScrapeStep = 'connecting' | 'page_loading' | 'parsing' | 'downloading' | 'saving' | 'completed' | 'failed';

export type TaskId = string;

export type Iso8601String = string;

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

export type JsonObject = { [key: string]: JsonValue };

export type ProtocolVersion = '1.0.0';

export interface ProductData {
  title: string;
  cover: ImageRef;
  gallery: ImageRef[];
  description: Description;
  detail_images: ImageRef[];
  skus: SkuItem[];
  sku_images: Record<string, ImageRef>;
  price: PriceRange;
  shop: ShopInfo;
}

export interface ImageRef {
  original_url: string;
  thumbnail_url: string;
  local_path: string | null;
}

export interface SkuItem {
  name: string;
  value: string;
  price: number;
  stock: number | null;
  image: ImageRef | null;
}

export interface PriceRange {
  min_price: number;
  max_price: number;
  currency: 'CNY';
}

export interface ShopInfo {
  name: string;
  url: string;
}

export interface Description {
  text: string;
  html: string | null;
  specs: SpecItem[];
}

export interface SpecItem {
  key: string;
  value: string;
}

export interface Task {
  id: TaskId;
  url: string;
  platform: Platform;
  item_id: string;
  title: string;
  status: TaskStatus;
  created_at: Iso8601String;
  folder_path: string | null;
}

export interface ImageRecord {
  id: number;
  task_id: TaskId;
  type: ImageType;
  original_url: string;
  local_path: string | null;
  width: number | null;
  height: number | null;
  size_bytes: number | null;
}

export interface TaskFilter {
  platform?: Platform;
  status?: TaskStatus;
  keyword?: string;
  item_id?: string;
  start_time?: Iso8601String;
  end_time?: Iso8601String;
  limit?: number;
  offset?: number;
}

export interface TaskSummary {
  id: TaskId;
  url: string;
  platform: Platform;
  item_id: string;
  title: string;
  status: TaskStatus;
  created_at: Iso8601String;
  folder_path: string | null;
  cover_path: string | null;
}

export interface TaskDetail {
  task: Task;
  product: ProductData | null;
  images: ImageRecord[];
  raw_path: string | null;
  meta_path: string | null;
  errors: ScrapeErrorInfo[];
}

export interface TaskResult {
  task_id: TaskId;
  status: TaskStatus;
  folder_path: string | null;
  product: ProductData | null;
  image_total: number;
  image_success: number;
  image_failed: number;
  errors: ScrapeErrorInfo[];
}

export interface ScrapeErrorInfo {
  step: ScrapeStep;
  code: string;
  message: string;
  recoverable: boolean;
}

export type ErrorCode =
  | 'CDP_CONNECT_FAILED'
  | 'CDP_TIMEOUT'
  | 'CDP_LAUNCH_TIMEOUT'
  | 'NO_BROWSER_FOUND'
  | 'URL_INVALID'
  | 'UNSUPPORTED_PLATFORM'
  | 'ITEM_ID_EXTRACT_FAILED'
  | 'DUPLICATE_TASK'
  | 'TASK_ALREADY_RUNNING'
  | 'TASK_NOT_FOUND'
  | 'TASK_CANCELLED'
  | 'PARSE_FAILED'
  | 'IMAGE_DOWNLOAD_FAILED'
  | 'STORAGE_FAILED'
  | 'PATH_NOT_ALLOWED'
  | 'CONFIG_INVALID'
  | 'UNKNOWN_ERROR';

export interface IpcError {
  code: ErrorCode;
  message: string;
  recoverable: boolean;
  step: ScrapeStep | null;
  details?: JsonObject;
}

export interface ConnectionInfo {
  port: number;
  endpoint: string;
  browser_version: string;
  state: ConnectionState;
}

export type ConnectionState =
  | { type: 'Disconnected' }
  | { type: 'Connecting' }
  | { type: 'Connected'; browser_version: string }
  | { type: 'Reconnecting'; attempt: number }
  | { type: 'Failed'; reason: string };

export interface BrowserLaunchCommand {
  os: 'macos' | 'windows';
  browser: 'chrome' | 'edge';
  command: string;
}

export interface TabInfo {
  id: string;
  title: string;
  url: string;
  type: string;
}

export interface AppConfig {
  cdp_port: number;
  storage_root: string;
  image_concurrency: number;
  browser_launch_commands: BrowserLaunchCommand[];
}

export interface MetaJsonDocument {
  version: ProtocolVersion;
  platform: Platform;
  item_id: string;
  scraped_at: Iso8601String;
  data: ProductData;
}

export interface RawJsonDocument {
  version: ProtocolVersion;
  platform: Platform;
  item_id: string;
  scraped_at: Iso8601String;
  url: string;
  raw_data: JsonObject;
  parser_errors: ScrapeErrorInfo[];
}

/** 单张图片的等比缩放结果详情。 */
export interface ResizeDetail {
  /** 图片文件路径 */
  path: string;
  /** 原始宽度 */
  original_width: number;
  /** 原始高度 */
  original_height: number;
  /** 缩放后宽度（skipped 时为 null） */
  new_width: number | null;
  /** 缩放后高度（skipped 时为 null） */
  new_height: number | null;
  /** 操作结果："resized" | "skipped" | "failed" */
  action: string;
  /** 失败原因（仅 action="failed" 时有值） */
  error: string | null;
}

/** 图片等比缩放任务的整体结果。 */
export interface ResizeResult {
  /** 扫描的图片总数 */
  total: number;
  /** 实际执行缩放的图片数 */
  resized: number;
  /** 跳过的图片数（已在限制内） */
  skipped: number;
  /** 处理失败的图片数 */
  failed: number;
  /** 每张图片的详细结果 */
  details: ResizeDetail[];
}
