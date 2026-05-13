# Protocol: Storage Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-08
- 依赖的真相源：`docs/PRD.md` 1.0.0（L1）第3.3节、`docs/ARCHITECTURE.md` 1.0.0（L2）第3.1/4.4/6节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type {
  ImageType,
  JsonObject,
  MetaJsonDocument,
  Platform,
  ProductData,
  RawJsonDocument,
  Task,
  TaskDetail,
  TaskFilter,
  TaskId,
  ScrapeErrorInfo,
  TaskStatus,
  TaskSummary
} from './data-models';

/** 存储引擎必须实现的统一接口。Rust 中为 trait 或 struct impl。 */
export interface StorageEngine {
  /** 初始化数据库和存储根目录。 */
  init(): Promise<void>;

  /** 创建任务记录，返回任务。force=true 时允许覆盖同平台同 item_id 的旧存档索引。 */
  create_task(url: string, platform: Platform, item_id: string, force?: boolean): Promise<Task>;

  /** 更新任务状态和元数据。 */
  update_task(task_id: TaskId, updates: TaskUpdate): Promise<void>;

  /** 保存商品元数据到 meta.json，返回相对存档根目录的路径 meta.json。 */
  save_meta(task_id: TaskId, product: ProductData): Promise<string>;

  /** 构造 meta.json 外层文档。 */
  build_meta_document(task_id: TaskId, product: ProductData): Promise<MetaJsonDocument>;

  /** 保存原始抓取数据到 raw.json，返回相对存档根目录的路径 raw.json；parser_errors 必须写入 RawJsonDocument。 */
  save_raw(task_id: TaskId, raw_data: JsonObject, parser_errors: ScrapeErrorInfo[]): Promise<string>;

  /** 构造 raw.json 外层文档；url/platform/item_id 从 task_id 对应 Task 读取，parser_errors 由 parser/scraper 传入。 */
  build_raw_document(task_id: TaskId, raw_data: JsonObject, parser_errors: ScrapeErrorInfo[]): Promise<RawJsonDocument>;

  /** 保存图片索引到 SQLite。 */
  index_image(image: ImageIndexInput): Promise<void>;

  /** 查询任务历史。 */
  query_tasks(filter: TaskFilter): Promise<TaskSummary[]>;

  /** 获取任务详情（含图片索引）。 */
  get_task_detail(task_id: TaskId): Promise<TaskDetail>;

  /** 检查 item_id 是否已存在（去重）。 */
  check_duplicate(platform: Platform, item_id: string): Promise<TaskId | null>;

  /** 打开本地文件夹；path 必须通过安全校验且位于 storage_root 或已知任务 folder_path 内。 */
  open_folder(path: string): Promise<boolean>;
}

/** 去重冲突信息。 */
export interface DuplicateTaskConflict {
  /** 冲突的既有任务 ID。 */
  existing_task_id: TaskId;
  /** 既有任务存档目录；不存在时为 null。 */
  existing_folder_path: string | null;
  /** 固定错误码。 */
  code: 'DUPLICATE_TASK';
}

/** 任务更新字段（部分更新）。 */
export interface TaskUpdate {
  status?: TaskStatus;
  title?: string;
  folder_path?: string | null;
}

/** 写入 images 表的输入。 */
export interface ImageIndexInput {
  task_id: TaskId;
  type: ImageType;
  original_url: string;
  local_path: string | null;
  width: number | null;
  height: number | null;
  size_bytes: number | null;
}

/** 存档目录结构描述。 */
export interface ArchiveStructure {
  /** 根目录名称：{platform}_{item_id}_{timestamp} */
  folder_name: string;

  /** 完整绝对路径。 */
  absolute_path: string;

  /** 子文件和目录。 */
  entries: ArchiveEntry[];
}

export interface ArchiveEntry {
  name: string;
  type: 'file' | 'directory';
  children?: ArchiveEntry[];
}
```

## 数据库 Schema

### tasks 表

```sql
CREATE TABLE IF NOT EXISTS tasks (
  id TEXT PRIMARY KEY,           -- 任务唯一标识
  url TEXT NOT NULL,             -- 用户输入的商品 URL
  platform TEXT NOT NULL,        -- 平台：taobao / tmall / jd
  item_id TEXT NOT NULL,         -- 平台商品 ID
  title TEXT NOT NULL DEFAULT '',-- 商品标题
  status TEXT NOT NULL,          -- pending / running / success / failed / partial / cancelled
  created_at TEXT NOT NULL,      -- ISO 8601 时间字符串
  folder_path TEXT,              -- 存档目录绝对路径
  UNIQUE(platform, item_id)      -- 去重约束：同一平台同一商品不重复抓取
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_platform ON tasks(platform);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_tasks_item_id ON tasks(item_id);
```

### images 表

```sql
CREATE TABLE IF NOT EXISTS images (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id TEXT NOT NULL,         -- 关联 tasks.id
  type TEXT NOT NULL,            -- cover / gallery / detail / sku
  original_url TEXT NOT NULL,    -- 原图 URL
  local_path TEXT,               -- 本地相对或绝对路径
  width INTEGER,                 -- 图片宽度（像素）
  height INTEGER,                -- 图片高度（像素）
  size_bytes INTEGER,            -- 文件大小（字节）
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_images_task_id ON images(task_id);
CREATE INDEX idx_images_type ON images(type);
```

### schema_migrations 表

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,    -- 单调递增 schema 版本
  applied_at TEXT NOT NULL        -- ISO 8601 UTC 秒级时间
);

INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (1, strftime('%Y-%m-%dT%H:%M:%SZ','now'));
```

- 当前 schema 版本为 `1`。
- 每次 schema 变更必须新增迁移，不得直接破坏用户既有 `index.db`。
- 应用启动时 `init()` 必须在业务查询前完成迁移检查。

## 存档目录结构

以商品为单位创建文件夹，命名规则：`{platform}_{item_id}_{timestamp}`

```
taobao_12345678_20260505T143022/
├── meta.json          # 标准化结构化元数据（ProductData JSON）
├── raw.json           # 原始抓取数据（完整解析结果，用于调试和重解析）
├── cover/             # 封面图
│   └── cover_001.jpg
├── gallery/           # 主图集
│   ├── main_001.jpg
│   ├── main_002.jpg
│   └── ...
├── detail/            # 详情页图片
│   ├── detail_001.jpg
│   ├── detail_002.jpg
│   └── ...
└── sku/               # SKU 变体图片
    ├── sku_红色.jpg
    ├── sku_蓝色.jpg
    └── ...
```

### 文件命名规范

| 类型 | 目录 | 文件名格式 | 示例 |
|------|------|-----------|------|
| 封面 | `cover/` | `cover_{序号}.{ext}` | `cover_001.jpg` |
| 主图 | `gallery/` | `main_{序号}.{ext}` | `main_001.jpg` |
| 详情图 | `detail/` | `detail_{序号}.{ext}` | `detail_001.jpg` |
| SKU 图 | `sku/` | `sku_{规格值}.{ext}` | `sku_红色.jpg` |

- 序号从 `001` 开始，三位零填充。
- `{ext}` 保持原始图片扩展名（`.jpg`、`.png`、`.webp` 等）。
- SKU 图片文件名中的规格值需进行文件系统安全处理（去除非法字符）。文件名安全规则必须跨 macOS/Windows 一致：
  - 禁止字符：`<`, `>`, `:`, `"`, `/`, `\\`, `|`, `?`, `*` 以及 ASCII 控制字符 `U+0000`-`U+001F`。
  - 禁止 Windows 保留设备名（大小写不敏感）：`CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9`；带扩展名的同名文件也禁止，例如 `CON.jpg`。
  - 文件名不得以空格或英文句点 `.` 结尾；空白折叠为单个 `_`，连续 `_` 可折叠为单个 `_`。
  - 清洗后为空或命中保留名时，使用稳定回退名：`sku_{序号三位零填充}`。
  - 单个文件名（不含目录）建议限制在 120 个 UTF-8 字节以内；截断时必须保留扩展名并避免产生空文件名。

### JSON 数据层规范

#### meta.json

`meta.json` 是标准化的商品结构化数据，字段定义与 `ProductData` 完全对齐。

```json
{
  "version": "1.0.0",
  "platform": "taobao",
  "item_id": "12345678",
  "scraped_at": "2026-05-05T14:30:22Z",
  "data": {
    "title": "...",
    "cover": { "original_url": "...", "thumbnail_url": "...", "local_path": "cover/cover_001.jpg" },
    "gallery": [...],
    "description": { "text": "...", "html": "...", "specs": [...] },
    "detail_images": [...],
    "skus": [...],
    "sku_images": { ... },
    "price": { "min_price": 99.0, "max_price": 129.0, "currency": "CNY" },
    "shop": { "name": "...", "url": "..." }
  }
}
```

#### raw.json

`raw.json` 是原始抓取数据，包含页面解析的完整结果，用于调试和历史重解析。

```json
{
  "version": "1.0.0",
  "platform": "taobao",
  "item_id": "12345678",
  "scraped_at": "2026-05-05T14:30:22Z",
  "url": "https://item.taobao.com/item.htm?id=12345678",
  "raw_data": {
    "g_config": { ... },
    "html_snapshot": "...",
    "evaluate_results": { ... }
  },
  "parser_errors": [
    { "step": "parsing", "code": "...", "message": "...", "recoverable": true }
  ]
}
```

## 约束

- 存储根目录可配置，默认：
  - macOS: `~/EGrab/`
  - Windows: `%USERPROFILE%\EGrab\`
- 数据库文件路径：
  - macOS: `~/Library/Application Support/com.egrab.app/index.db`
  - Windows: `%APPDATA%\com.egrab.app\index.db`
- `tasks` 表通过 `(platform, item_id)` 唯一约束实现去重；`create_task(..., force=false)` 遇到重复时必须返回 `DUPLICATE_TASK` 错误并携带 `DuplicateTaskConflict`；`force=true` 时允许重新抓取。受当前 schema 唯一约束限制，强制重抓必须在事务内替换旧任务索引（删除旧 `images` 索引、更新或重建 `tasks` 记录），并保证文件系统不会误删非本应用目录。
- `meta.json` 中的 `local_path` 使用相对路径（相对于存档根目录），便于迁移。
- `save_meta` 返回值固定为相对路径 `meta.json`；`save_raw` 返回值固定为相对路径 `raw.json`；实际绝对路径由任务 `folder_path` 拼接得到。
- `raw.json` 必须完整保留，即使解析失败也应写入，确保可重解析；`raw_data` Rust 类型为 `HashMap<String, serde_json::Value>` 或等价结构；`parser_errors` 必须来自解析/抓取流程，不得静默丢弃。
- 图片下载失败时，`images.local_path` 为 `null`，`tasks.status` 可为 `partial`。
- 所有时间字段使用 ISO 8601 UTC 秒级精度格式字符串（例如 `2026-05-05T14:30:22Z`）。
- 路径字段必须为本地绝对路径或相对路径，不得包含远程上传地址。
- 所有由商品标题、SKU 规格值、平台原始字段派生出的目录名或文件名，必须先按“文件命名规范”的跨平台安全规则清洗；清洗不得改变 `ProductData` / `raw.json` 中保留的原始业务文本。
- `open_folder(path)` 安全要求：实现必须 canonicalize 路径；拒绝空路径、远程 URL、包含 NUL 字符的路径、解析后不在 `storage_root` 下且不等于已知任务 `folder_path` 的路径；不得跟随符号链接逃逸出允许根目录；失败时返回 `PATH_NOT_ALLOWED`。
- 并发控制：MVP 单商品抓取同一时刻最多 1 个活动写入任务；SQLite 写操作必须串行化或使用事务，避免 `tasks`、`images` 与文件系统状态不一致。
- 取消任务清理：`cancel_scrape` 后，已写入的 `raw.json` 可保留用于调试；未完成图片文件可删除或保留，但必须在 `TaskResult.errors` 中说明，最终任务状态为 `cancelled`，不得误标记为 `success`。

## 示例

```json
{
  "folder_name": "taobao_12345678_20260505T143022",
  "absolute_path": "/Users/alice/EGrab/taobao_12345678_20260505T143022",
  "entries": [
    { "name": "meta.json", "type": "file" },
    { "name": "raw.json", "type": "file" },
    {
      "name": "cover",
      "type": "directory",
      "children": [{ "name": "cover_001.jpg", "type": "file" }]
    },
    {
      "name": "gallery",
      "type": "directory",
      "children": [
        { "name": "main_001.jpg", "type": "file" },
        { "name": "main_002.jpg", "type": "file" }
      ]
    },
    {
      "name": "detail",
      "type": "directory",
      "children": [
        { "name": "detail_001.jpg", "type": "file" },
        { "name": "detail_002.jpg", "type": "file" }
      ]
    },
    {
      "name": "sku",
      "type": "directory",
      "children": [
        { "name": "sku_红色.jpg", "type": "file" },
        { "name": "sku_蓝色.jpg", "type": "file" }
      ]
    }
  ]
}
```
