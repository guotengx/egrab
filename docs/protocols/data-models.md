# Protocol: Data Models

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-08
- 依赖的真相源：`docs/PRD.md` 1.0.0（L1）、`docs/ARCHITECTURE.md` 1.0.0（L2）、`AGENTS.md` 全局一致性铁律

## 类型定义

以下 TypeScript 定义是 Rust/TypeScript 双端共享语义描述。Rust 实现时应使用 `serde` 保持 JSON 字段名不变。

**JSON/serde 约定**：所有跨 IPC、事件、`meta.json`、`raw.json` 的 JSON 字段名均使用本文档中出现的 `snake_case`；Rust 端不得通过 `rename_all = "camelCase"` 改名。判别联合（如 `ConnectionState`）使用 `#[serde(tag = "type")]`，variant 名称保持 `Disconnected`、`Connecting`、`Connected`、`Reconnecting`、`Failed`。

```ts
/** MVP 已知平台标识。 */
export type KnownPlatform = 'taobao' | 'tmall' | 'jd';

/** 平台标识。MVP 仅允许 KnownPlatform；为后续扩展保留字符串承载能力，但实现不得在未获 PRD/ARCHITECTURE 变更前启用新平台。 */
export type Platform = KnownPlatform | (string & { readonly __platform_extension?: never });

/** 抓取任务状态。 */
export type TaskStatus = 'pending' | 'running' | 'success' | 'failed' | 'partial' | 'cancelled';

/** 图片资源类型，对应存档子目录。 */
export type ImageType = 'cover' | 'gallery' | 'detail' | 'sku';

/** 抓取步骤，用于进度展示。 */
export type ScrapeStep = 'connecting' | 'page_loading' | 'parsing' | 'downloading' | 'saving' | 'completed' | 'failed';

/** 任务唯一标识，Rust 中为 String。 */
export type TaskId = string;

/** ISO 8601 UTC 时间字符串，格式固定为 YYYY-MM-DDTHH:mm:ssZ；存档目录时间戳使用 YYYYMMDDTHHmmss。 */
export type Iso8601String = string;

/** JSON 值；Rust 对应 serde_json::Value。 */
export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

/** JSON 对象；Rust 对应 HashMap<String, serde_json::Value> 或 serde_json::Map<String, Value>。 */
export type JsonObject = { [key: string]: JsonValue };

/** 协议版本号。 */
export type ProtocolVersion = '1.0.0';

/** 商品结构化数据；字段名必须与 PRD 3.1.2 完全一致。 */
export interface ProductData {
  /** 商品标题：主标题 + 副标题；必填，非空字符串。 */
  title: string;
  /** 封面图：商品主图第一张；必填。 */
  cover: ImageRef;
  /** 商品主图集：轮播图所有图片；必填，可为空数组。 */
  gallery: ImageRef[];
  /** 详情文案：描述文字、卖点、规格参数表；必填。 */
  description: Description;
  /** 详情图片：详情页长图内容；必填，可为空数组。 */
  detail_images: ImageRef[];
  /** SKU 信息：规格变体、价格、库存；必填，可为空数组。 */
  skus: SkuItem[];
  /** SKU 图片：key 为规格值或平台可稳定识别的 SKU key；必填，可为空对象。 */
  sku_images: Record<string, ImageRef>;
  /** 商品价格区间；必填。 */
  price: PriceRange;
  /** 店铺信息；必填。 */
  shop: ShopInfo;
}

/** 图片引用；下载前 local_path 为 null，下载后为相对或绝对本地路径。 */
export interface ImageRef {
  /** 原图 URL：必须尽力去除压缩/裁剪参数。 */
  original_url: string;
  /** 页面显示缩略图 URL；无缩略图时可与 original_url 相同。 */
  thumbnail_url: string;
  /** 下载后的本地路径；未下载或失败时为 null。 */
  local_path: string | null;
}

/** 单个 SKU 规格项。 */
export interface SkuItem {
  /** 规格名，例如“颜色”“尺码”。 */
  name: string;
  /** 规格值，例如“红色”“XL”。 */
  value: string;
  /** 此规格价格，单位为 currency 指定币种。 */
  price: number;
  /** 库存；平台未提供时为 null。 */
  stock: number | null;
  /** 此规格关联图片；没有时为 null。 */
  image: ImageRef | null;
}

/** 商品价格区间。 */
export interface PriceRange {
  /** 最低价格；必须 >= 0。 */
  min_price: number;
  /** 最高价格；必须 >= min_price。 */
  max_price: number;
  /** 币种；MVP 固定为 CNY。 */
  currency: 'CNY';
}

/** 店铺信息。 */
export interface ShopInfo {
  /** 店铺名称；必填，非空字符串。 */
  name: string;
  /** 店铺链接；必填，应为 http/https URL。 */
  url: string;
}

/** 商品详情文案。 */
export interface Description {
  /** 纯文本描述；必填，可为空字符串。 */
  text: string;
  /** 原始 HTML；未保留时为 null。 */
  html: string | null;
  /** 规格参数表；必填，可为空数组。 */
  specs: SpecItem[];
}

/** 规格参数键值对。 */
export interface SpecItem {
  /** 参数名。 */
  key: string;
  /** 参数值。 */
  value: string;
}

/** SQLite tasks 表对应的任务记录。 */
export interface Task {
  /** 任务 ID，系统生成，唯一。格式：task_YYYYMMDD_HHmmss_六位递增或随机后缀；必须全局唯一且稳定。 */
  id: TaskId;
  /** 用户输入的商品 URL。 */
  url: string;
  /** 平台标识。 */
  platform: Platform;
  /** 平台商品 ID。 */
  item_id: string;
  /** 商品标题；解析前可为空字符串。 */
  title: string;
  /** 任务状态。 */
  status: TaskStatus;
  /** 创建时间，ISO 8601 UTC 字符串，秒级精度。 */
  created_at: Iso8601String;
  /** 商品存档文件夹路径；未写入前可为 null。 */
  folder_path: string | null;
}

/** SQLite images 表对应的图片索引。 */
export interface ImageRecord {
  /** 图片记录 ID，SQLite INTEGER PRIMARY KEY AUTOINCREMENT；JSON 中为安全整数。 */
  id: number;
  /** 所属任务 ID。 */
  task_id: TaskId;
  /** 图片类型。 */
  type: ImageType;
  /** 原图 URL。 */
  original_url: string;
  /** 本地路径；下载失败时可为 null。 */
  local_path: string | null;
  /** 图片宽度，非负整数；未知时为 null。 */
  width: number | null;
  /** 图片高度，非负整数；未知时为 null。 */
  height: number | null;
  /** 文件大小字节数，非负整数；未知或失败时为 null。 */
  size_bytes: number | null;
}

/** 历史查询过滤条件。 */
export interface TaskFilter {
  /** 平台过滤；未指定则不过滤。 */
  platform?: Platform;
  /** 状态过滤；未指定则不过滤。 */
  status?: TaskStatus;
  /** 关键词，匹配 title/url/item_id；未指定则不过滤。 */
  keyword?: string;
  /** 平台商品 ID 精确匹配；未指定则不过滤。 */
  item_id?: string;
  /** 起始创建时间（含），ISO 8601 UTC 秒级精度；未指定则无下限。 */
  start_time?: Iso8601String;
  /** 结束创建时间（含），ISO 8601 UTC 秒级精度；未指定则无上限。 */
  end_time?: Iso8601String;
  /** 返回数量上限；默认由实现决定。 */
  limit?: number;
  /** 偏移量；默认 0。 */
  offset?: number;
}

/** 历史列表摘要。 */
export interface TaskSummary {
  /** 任务 ID。 */
  id: TaskId;
  /** 商品 URL。 */
  url: string;
  /** 平台标识。 */
  platform: Platform;
  /** 商品 ID。 */
  item_id: string;
  /** 商品标题。 */
  title: string;
  /** 状态。 */
  status: TaskStatus;
  /** 创建时间 ISO 8601 UTC 秒级精度。 */
  created_at: Iso8601String;
  /** 存档目录。 */
  folder_path: string | null;
  /** 封面本地路径或缩略图路径；无时为 null。 */
  cover_path: string | null;
}

/** 任务详情。 */
export interface TaskDetail {
  /** 任务记录。 */
  task: Task;
  /** 标准化商品数据；任务失败且未生成时为 null。 */
  product: ProductData | null;
  /** 图片索引列表。 */
  images: ImageRecord[];
  /** raw.json 路径；未生成时为 null。 */
  raw_path: string | null;
  /** meta.json 路径；未生成时为 null。 */
  meta_path: string | null;
  /** 错误和警告列表。 */
  errors: ScrapeErrorInfo[];
}

/** 抓取完成结果。 */
export interface TaskResult {
  /** 任务 ID。 */
  task_id: TaskId;
  /** 最终任务状态：success/failed/partial/cancelled。 */
  status: TaskStatus;
  /** 存档目录；失败且未写入时为 null。 */
  folder_path: string | null;
  /** 解析出的商品数据；失败且未解析时为 null。 */
  product: ProductData | null;
  /** 图片总数。 */
  image_total: number;
  /** 图片成功下载数。 */
  image_success: number;
  /** 图片失败下载数。 */
  image_failed: number;
  /** 错误和警告列表。 */
  errors: ScrapeErrorInfo[];
}

/** 错误或警告信息。 */
export interface ScrapeErrorInfo {
  /** 发生阶段。 */
  step: ScrapeStep;
  /** 机器可读错误码。 */
  code: string;
  /** 用户可读消息。 */
  message: string;
  /** 是否可恢复；图片单张失败通常为 true。 */
  recoverable: boolean;
}

/** 统一错误码。实现可增加更细分 code，但必须落入这些类别之一。 */
export type ErrorCode =
  | 'CDP_CONNECT_FAILED'
  | 'NO_BROWSER_FOUND'
  | 'CDP_LAUNCH_TIMEOUT'
  | 'CDP_TIMEOUT'
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

/** IPC 错误返回格式；Rust Tauri command 错误应序列化为此结构，前端不得依赖裸字符串。 */
export interface IpcError {
  /** 机器可读错误码。 */
  code: ErrorCode;
  /** 用户可读错误消息。 */
  message: string;
  /** 错误是否可恢复。true 表示可重试、可降级或不会破坏任务一致性。 */
  recoverable: boolean;
  /** 发生阶段；无法归类时为 null。 */
  step: ScrapeStep | null;
  /** 可选调试细节；不得包含账号密码或 Cookie。 */
  details?: JsonObject;
}

/** CDP 连接信息。 */
export interface ConnectionInfo {
  /** CDP 端口。 */
  port: number;
  /** WebSocket endpoint，例如 ws://127.0.0.1:9222。 */
  endpoint: string;
  /** 浏览器版本；仅当 state.type 为 Connected 时有意义。 */
  browser_version: string;
  /** 当前连接状态。 */
  state: ConnectionState;
}

/** CDP 连接状态，与 ARCHITECTURE 4.2 对齐。 */
export type ConnectionState =
  | { type: 'Disconnected' }
  | { type: 'Connecting' }
  | { type: 'Connected'; browser_version: string }
  | { type: 'Reconnecting'; attempt: number }
  | { type: 'Failed'; reason: string };

/** 浏览器启动命令参考，用于设置界面展示和一键复制。 */
export interface BrowserLaunchCommand {
  /** 平台：macos 或 windows。 */
  os: 'macos' | 'windows';
  /** 浏览器：Chrome 或 Edge。 */
  browser: 'chrome' | 'edge';
  /** 用户可复制的启动命令。 */
  command: string;
}

/** 浏览器 Tab 信息。 */
export interface TabInfo {
  /** CDP target id。 */
  id: string;
  /** Tab 标题。 */
  title: string;
  /** 当前 URL。 */
  url: string;
  /** Tab 类型，通常为 page。 */
  type: string;
}

/** 应用配置。 */
export interface AppConfig {
  /** CDP 端口，默认 9222。 */
  cdp_port: number;
  /** 存储根目录，默认 macOS ~/EGrab/，Windows %USERPROFILE%\\EGrab\\。 */
  storage_root: string;
  /** 图片下载并发数，默认 3，最大 10。 */
  image_concurrency: number;
  /** 浏览器启动命令参考；由 config 模块按操作系统提供，前端设置界面只读展示。 */
  browser_launch_commands: BrowserLaunchCommand[];
}

/** meta.json 文件外层结构。 */
export interface MetaJsonDocument {
  /** 文档版本。 */
  version: ProtocolVersion;
  /** 平台标识。 */
  platform: Platform;
  /** 平台商品 ID。 */
  item_id: string;
  /** 抓取完成或写入时间，ISO 8601 UTC 秒级精度。 */
  scraped_at: Iso8601String;
  /** 标准化商品结构化数据。 */
  data: ProductData;
}

/** raw.json 文件外层结构。 */
export interface RawJsonDocument {
  /** 文档版本。 */
  version: ProtocolVersion;
  /** 平台标识。 */
  platform: Platform;
  /** 平台商品 ID。 */
  item_id: string;
  /** 抓取完成或写入时间，ISO 8601 UTC 秒级精度。 */
  scraped_at: Iso8601String;
  /** 原始商品 URL。 */
  url: string;
  /** 原始抓取数据；Rust 对应 HashMap<String, serde_json::Value>。 */
  raw_data: JsonObject;
  /** 解析错误和警告。 */
  parser_errors: ScrapeErrorInfo[];
}
```

## 约束
- `ProductData` 的九个顶层字段 `title`, `cover`, `gallery`, `description`, `detail_images`, `skus`, `sku_images`, `price`, `shop` 全部必填，拼写不可改变。
- 协议版本兼容策略：`ProtocolVersion` 当前为 `1.0.0`；同一 major 版本内只能新增可选字段或新增错误码，不得删除字段、改名或改变既有字段类型；破坏性变更必须提升 major 版本，并按 AGENTS.md 变更传导协议更新 contract、protocols、`src/protocols/`、实现和测试。
- `ImageRef.original_url` 必须尽力为原始分辨率图片 URL；淘宝需去除 `_xxx.jpg` 尺寸标记，京东需去除 `s800x800_jfs` 等尺寸前缀；失败时可降级为页面显示尺寸并记录错误。
- `PriceRange.currency` MVP 固定为 `CNY`；`min_price >= 0`，`max_price >= min_price`。
- `SkuItem.price >= 0`；`SkuItem.stock` 为 `null` 或非负整数。
- 所有 TypeScript `number` 若注释为整数，Rust 必须使用整数类型：`port: u16`、`stock: Option<u32>`、`width/height: Option<u32>`、`size_bytes: Option<u64>`、`ImageRecord.id: i64`；价格使用浮点或十进制定点，JSON 中表现为 number。
- `ConnectionState.Reconnecting.attempt` 范围为 1 到 3；CDP 连接仅允许 localhost (`127.0.0.1`)。
- `AppConfig.image_concurrency` 默认 3，最大 10，最小 1；`AppConfig.cdp_port` 默认 9222。
- 时间字段使用 ISO 8601 UTC 秒级精度字符串；路径字段必须为本地路径，不得为远程上传地址。
- `ConnectionState` JSON 序列化必须采用 `{ "type": "Connected", ... }` 形式；不得使用外部 tag、数组或小写 variant。
- `TaskId` 必须全局唯一；测试可使用固定样例 `task_20260508_000001`。
- `recoverable` 语义：true 表示调用方可以继续流程、重试或降级；false 表示当前命令或任务无法按原目标继续。

## 示例

```json
{
  "title": "示例商品 主标题 + 副标题",
  "cover": {
    "original_url": "https://img.example.com/item/cover.jpg",
    "thumbnail_url": "https://img.example.com/item/cover_400x400.jpg",
    "local_path": "cover/cover_001.jpg"
  },
  "gallery": [
    {
      "original_url": "https://img.example.com/item/main1.jpg",
      "thumbnail_url": "https://img.example.com/item/main1_400x400.jpg",
      "local_path": "gallery/main_001.jpg"
    }
  ],
  "description": {
    "text": "商品卖点与规格描述",
    "html": null,
    "specs": [{ "key": "材质", "value": "棉" }]
  },
  "detail_images": [
    {
      "original_url": "https://img.example.com/item/detail1.jpg",
      "thumbnail_url": "https://img.example.com/item/detail1.jpg",
      "local_path": "detail/detail_001.jpg"
    }
  ],
  "skus": [
    {
      "name": "颜色",
      "value": "红色",
      "price": 99.0,
      "stock": 100,
      "image": {
        "original_url": "https://img.example.com/item/sku-red.jpg",
        "thumbnail_url": "https://img.example.com/item/sku-red_100x100.jpg",
        "local_path": "sku/sku_红色.jpg"
      }
    }
  ],
  "sku_images": {
    "红色": {
      "original_url": "https://img.example.com/item/sku-red.jpg",
      "thumbnail_url": "https://img.example.com/item/sku-red_100x100.jpg",
      "local_path": "sku/sku_红色.jpg"
    }
  },
  "price": { "min_price": 99.0, "max_price": 129.0, "currency": "CNY" },
  "shop": { "name": "示例店铺", "url": "https://shop.example.com" }
}
```
