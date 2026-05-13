# EGrab - 技术架构文档

> 版本: 1.0.0  
> 最后更新: 2026-05-05  
> 状态: 已确认  
> 权威等级: L2（技术真相源）

---

## 1. 技术栈总览

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| 桌面框架 | Tauri 2.x | Rust后端 + 系统WebView前端 |
| 前端框架 | Svelte 5 + TypeScript | 编译型UI框架，体积小性能好 |
| 前端构建 | Vite | 开发热更新 + 生产构建 |
| 后端语言 | Rust | Tauri核心，负责系统调用和重计算 |
| CDP通信 | chromiumoxide (Rust crate) | Rust原生CDP客户端库 |
| 数据库 | SQLite (via rusqlite) | 嵌入式数据库，无需额外安装 |
| 序列化 | serde + serde_json | Rust标准JSON序列化 |
| HTTP客户端 | reqwest | 图片下载用，支持并发 |
| 样式方案 | Tailwind CSS 4 | 原子化CSS |
| 跨平台打包 | tauri-bundler | 输出 .dmg(mac) / .msi(win) |

---

## 2. 系统架构图

```
┌─────────────────────────────────────────────────────────┐
│                    EGrab Client                          │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │              Frontend (Svelte + TS)              │    │
│  │                                                 │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │ HomePage │ │ Progress │ │ ArchivePage  │   │    │
│  │  │          │ │   View   │ │              │   │    │
│  │  └──────────┘ └──────────┘ └──────────────┘   │    │
│  │                                                 │    │
│  │  ┌─────────────────────────────────────────┐   │    │
│  │  │         Tauri IPC Bridge (invoke)       │   │    │
│  │  └─────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────┘    │
│                          │                               │
│                    Tauri IPC                             │
│                          │                               │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Backend (Rust)                      │    │
│  │                                                 │    │
│  │  ┌───────────┐ ┌────────────┐ ┌────────────┐  │    │
│  │  │    CDP    │ │  Scraper   │ │  Storage   │  │    │
│  │  │  Manager  │ │   Engine   │ │   Engine   │  │    │
│  │  └─────┬─────┘ └──────┬─────┘ └──────┬─────┘  │    │
│  │        │               │              │         │    │
│  │  ┌─────┴─────┐ ┌──────┴──────┐ ┌────┴──────┐ │    │
│  │  │  Browser  │ │  Platform   │ │  SQLite   │ │    │
│  │  │ Connector │ │  Parsers    │ │    DB     │ │    │
│  │  └───────────┘ │             │ └───────────┘ │    │
│  │                 │ ┌─────────┐ │               │    │
│  │                 │ │ Taobao  │ │               │    │
│  │                 │ ├─────────┤ │               │    │
│  │                 │ │   JD    │ │               │    │
│  │                 │ └─────────┘ │               │    │
│  │                 └─────────────┘               │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
         │
    CDP WebSocket
         │
┌────────┴────────┐
│  Chrome/Edge    │
│  (用户浏览器)    │
│  :9222          │
└─────────────────┘
```

---

## 3. 模块划分

### 3.1 后端模块（Rust / src-tauri/）

| 模块 | 路径 | 职责 |
|------|------|------|
| `cdp` | `src-tauri/src/cdp/` | CDP连接管理、WebSocket通信、页面导航 |
| `scraper` | `src-tauri/src/scraper/` | 抓取引擎核心，协调解析器和下载器 |
| `parser` | `src-tauri/src/parser/` | 平台解析器（每个平台一个子模块） |
| `downloader` | `src-tauri/src/downloader/` | 图片批量下载器（并发控制、重试） |
| `storage` | `src-tauri/src/storage/` | 存档引擎（SQLite + JSON + 文件系统） |
| `models` | `src-tauri/src/models/` | 全局数据模型定义（与PRD 3.1.2对齐） |
| `commands` | `src-tauri/src/commands/` | Tauri IPC命令定义（前端可调用的接口） |
| `config` | `src-tauri/src/config/` | 应用配置管理 |

### 3.2 前端模块（Svelte / src/）

| 模块 | 路径 | 职责 |
|------|------|------|
| `pages` | `src/pages/` | 页面组件（Home, Progress, Archive, Settings） |
| `components` | `src/components/` | 可复用UI组件 |
| `stores` | `src/stores/` | Svelte状态管理（连接状态、任务状态等） |
| `services` | `src/services/` | Tauri IPC调用封装 |
| `types` | `src/types/` | TypeScript类型定义（与Rust models对齐） |

### 3.3 共享协议层

| 路径 | 职责 | 生成者 |
|------|------|--------|
| `docs/protocols/` | 接口协议文档（人类可读） | pre agent |
| `src/protocols/` | 代码级类型定义（编译器可检查） | architect agent |

---

## 4. 核心流程

### 4.1 抓取流程时序

```
Frontend                Backend(Rust)              Chrome
   │                        │                        │
   │─── invoke:start_scrape(url, force?) ──→│        │
   │                        │                        │
   │                        │── CDP:connect ────────→│
   │                        │←── connected ──────────│
   │                        │                        │
   │                        │── CDP:navigate(url) ──→│
   │                        │←── page_loaded ────────│
   │                        │                        │
   │←── event:progress(10%) │                        │
   │                        │                        │
   │                        │── CDP:evaluate(js) ───→│
   │                        │←── dom_data ───────────│
   │                        │                        │
   │←── event:progress(40%) │                        │
   │                        │                        │
   │                        │── parse(platform, data)│
   │                        │── download_images() ──→│ (HTTP直连图片CDN)
   │                        │                        │
   │←── event:progress(80%) │                        │
   │                        │                        │
   │                        │── storage:save()       │
   │                        │                        │
   │←── event:complete(result)│                      │
   │                        │                        │
```

### 4.2 CDP 连接管理

```rust
// 伪代码 - CDP连接生命周期
struct CdpManager {
    endpoint: String,          // ws://127.0.0.1:9222
    browser: Option<Browser>,
    connection_state: ConnectionState,
}

enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { browser_version: String },
    Reconnecting { attempt: u8 },
    Failed { reason: String },
}
```

### 4.3 平台解析器接口

```rust
// 所有平台解析器必须实现的trait
trait PlatformParser {
    fn platform_id(&self) -> &str;
    fn can_handle(&self, url: &str) -> bool;
    fn extract_item_id(&self, url: &str) -> Result<String>;
    async fn parse(&self, page: &Page) -> Result<ProductData>;
}
```

### 4.4 数据模型（全局统一）

```rust
// 与 PRD 3.1.2 严格对齐的字段命名
struct ProductData {
    title: String,
    cover: ImageRef,
    gallery: Vec<ImageRef>,
    description: Description,
    detail_images: Vec<ImageRef>,
    skus: Vec<SkuItem>,
    sku_images: HashMap<String, ImageRef>,
    price: PriceRange,
    shop: ShopInfo,
}

struct ImageRef {
    original_url: String,      // 原图URL（去压缩参数后）
    thumbnail_url: String,     // 页面显示的缩略图URL
    local_path: Option<String>, // 下载后的本地路径
}

struct SkuItem {
    name: String,              // 规格名（如"颜色"）
    value: String,             // 规格值（如"红色"）
    price: f64,
    stock: Option<u32>,
    image: Option<ImageRef>,
}

struct PriceRange {
    min_price: f64,
    max_price: f64,
    currency: String,          // "CNY"
}

struct ShopInfo {
    name: String,
    url: String,
}

struct Description {
    text: String,              // 纯文本描述
    html: Option<String>,      // 原始HTML（可选保留）
    specs: Vec<SpecItem>,      // 规格参数表
}

struct SpecItem {
    key: String,
    value: String,
}
```

---

## 5. IPC 接口设计

### 5.1 Tauri Commands（前端→后端）

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `cdp_connect` | `port: u16` | `ConnectionInfo` | 连接CDP |
| `cdp_disconnect` | - | `bool` | 断开CDP |
| `cdp_status` | - | `ConnectionState` | 查询连接状态 |
| `cdp_list_tabs` | - | `Vec<TabInfo>` | 列出浏览器标签 |
| `start_scrape` | `url: String, force: Option<bool>` | `TaskId` | 开始抓取任务；force=true 时强制重新抓取已存在商品 |
| `cancel_scrape` | `task_id: String` | `bool` | 取消抓取 |
| `get_task_history` | `filter: TaskFilter` | `Vec<TaskSummary>` | 查询历史 |
| `get_task_detail` | `task_id: String` | `TaskDetail` | 获取任务详情 |
| `open_folder` | `path: String` | `bool` | 打开本地文件夹 |
| `get_config` | - | `AppConfig` | 获取配置 |
| `set_config` | `config: AppConfig` | `bool` | 保存配置 |

### 5.2 Tauri Events（后端→前端）

| 事件 | Payload | 说明 |
|------|---------|------|
| `scrape:progress` | `{ task_id, percent, step, message }` | 抓取进度 |
| `scrape:complete` | `{ task_id, result: TaskResult }` | 抓取完成 |
| `scrape:error` | `{ task_id, error, recoverable }` | 抓取错误 |
| `cdp:state_changed` | `ConnectionState` | CDP连接状态变更 |

---

## 6. 目录结构

```
egrab/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # 库入口
│   │   ├── cdp/                 # CDP连接管理
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs       # 连接管理器
│   │   │   └── types.rs         # CDP相关类型
│   │   ├── scraper/             # 抓取引擎
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs        # 抓取协调器
│   │   │   └── task.rs          # 任务状态管理
│   │   ├── parser/              # 平台解析器
│   │   │   ├── mod.rs           # PlatformParser trait定义
│   │   │   ├── taobao.rs        # 淘宝/天猫解析器
│   │   │   ├── jd.rs            # 京东解析器
│   │   │   └── utils.rs         # 解析工具函数
│   │   ├── downloader/          # 图片下载器
│   │   │   ├── mod.rs
│   │   │   └── image.rs         # 并发图片下载
│   │   ├── storage/             # 存储引擎
│   │   │   ├── mod.rs
│   │   │   ├── database.rs      # SQLite操作
│   │   │   ├── filesystem.rs    # 文件系统操作
│   │   │   └── schema.sql       # 建表SQL
│   │   ├── models/              # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── product.rs       # ProductData等核心模型
│   │   │   └── task.rs          # Task相关模型
│   │   ├── commands/            # Tauri IPC命令
│   │   │   ├── mod.rs
│   │   │   ├── cdp_commands.rs
│   │   │   ├── scrape_commands.rs
│   │   │   └── config_commands.rs
│   │   └── config/              # 应用配置
│   │       ├── mod.rs
│   │       └── app_config.rs
│   └── icons/                   # 应用图标
├── src/                          # Svelte 前端
│   ├── App.svelte
│   ├── main.ts
│   ├── pages/
│   │   ├── Home.svelte          # 主页（URL输入+连接状态）
│   │   ├── Progress.svelte      # 抓取进度
│   │   ├── Archive.svelte       # 存档浏览
│   │   └── Settings.svelte      # 设置
│   ├── components/
│   │   ├── StatusBar.svelte     # CDP状态栏
│   │   ├── TaskCard.svelte      # 任务卡片
│   │   ├── ProgressBar.svelte   # 进度条
│   │   └── UrlInput.svelte      # URL输入框
│   ├── stores/
│   │   ├── connection.ts        # CDP连接状态store
│   │   ├── tasks.ts             # 抓取任务状态store
│   │   └── config.ts            # 配置store
│   ├── services/
│   │   ├── ipc.ts               # Tauri invoke封装
│   │   └── events.ts            # Tauri event监听封装
│   └── types/
│       ├── product.ts           # 商品数据类型（与Rust models对齐）
│       ├── task.ts              # 任务类型
│       └── config.ts            # 配置类型
├── src/protocols/                # 代码级接口定义（architect生成）
│   ├── README.md
│   └── ...
├── docs/                         # 文档
│   ├── PRD.md                   # 产品需求文档
│   ├── ARCHITECTURE.md          # 本文件
│   ├── pre-mandate.md           # pre agent任务指令书
│   ├── contract-*.md            # 各Agent专属约束（pre生成）
│   └── protocols/               # 接口协议文档（pre生成）
├── AGENTS.md                    # 全局Agent协作规范
├── STATUS.md                    # 项目状态追踪
├── HISTORY.md                   # 压缩历史记录
├── TECH_BOARD.md                # 技术看板
├── opencode.json                # 多Agent配置
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

---

## 7. 技术决策记录

### 7.1 为什么选 Tauri 而不是 Electron

| 维度 | Tauri | Electron |
|------|-------|----------|
| 打包体积 | ~10MB | ~150MB |
| 内存占用 | ~50MB | ~200MB+ |
| 安全性 | Rust内存安全 | Node.js |
| CDP集成 | chromiumoxide (原生Rust) | puppeteer需要额外Chromium |
| 系统WebView | 复用系统WebView | 自带Chromium |

### 7.2 为什么选 chromiumoxide

- 纯Rust实现的CDP客户端，与Tauri后端无缝集成
- 无需额外进程，直接WebSocket连接用户浏览器
- 类型安全，编译期检查CDP协议调用

### 7.3 为什么选 SQLite 而不是纯文件

- 支持复杂查询（按时间、平台、关键词检索）
- 去重检测高效
- 单文件数据库，便于备份迁移
- rusqlite 集成成熟

### 7.4 为什么前端选 Svelte 而不是 React/Vue

- 编译型框架，无虚拟DOM开销
- 打包体积极小，适合Tauri追求极致体积的理念
- Tauri官方模板原生支持
- 语法简洁，开发效率高

---

## 8. 跨平台差异处理

| 差异点 | macOS | Windows |
|--------|-------|---------|
| WebView引擎 | WebKit | WebView2 (Chromium) |
| Chrome默认路径 | `/Applications/Google Chrome.app` | `C:\Program Files\Google\Chrome\Application\chrome.exe` |
| CDP启动命令 | `open -a "Google Chrome" --args --remote-debugging-port=9222` | `chrome.exe --remote-debugging-port=9222` |
| 存储默认路径 | `~/EGrab/` | `%USERPROFILE%\EGrab\` |
| 数据库路径 | `~/Library/Application Support/com.egrab.app/index.db` | `%APPDATA%\com.egrab.app\index.db` |

---

## 9. 错误处理策略

| 错误类型 | 处理方式 | 用户提示 |
|---------|---------|---------|
| CDP连接失败 | 重试3次后报错 | 提示检查浏览器是否启动CDP |
| 页面加载超时 | 30s超时，允许重试 | 提示网络问题或页面异常 |
| 解析失败 | 记录raw数据，标记partial | 提示平台可能改版 |
| 图片下载失败 | 单张失败不中断，记录错误 | 完成后汇总失败数量 |
| 存储空间不足 | 检测后拒绝执行 | 提示清理空间 |

---

## 10. 全局一致性声明

本文档是 EGrab 项目的 **L2 技术真相源**。

### 10.1 命名一致性约束

以下命名在全系统中具有唯一确定含义，所有模块必须统一使用：

- **模块名**：cdp, scraper, parser, downloader, storage, models, commands, config
- **数据模型字段名**：与 PRD 3.1.2 节严格对齐（title, cover, gallery, description, detail_images, skus, sku_images, price, shop）
- **IPC命令名**：第5节定义为最终命名，前后端必须一致
- **事件名**：`scrape:progress`, `scrape:complete`, `scrape:error`, `cdp:state_changed`

### 10.2 类型对齐约束

- Rust `src-tauri/src/models/` 中的 struct 定义为**类型权威**
- TypeScript `src/types/` 必须与 Rust models 一一对应
- `src/protocols/` 是 architect 从本文档生成的桥接类型定义

### 10.3 变更传导

如果本文档发生变更：
1. pre 必须重新检查并更新受影响的 contract-*.md 和 protocols/*.md
2. architect 必须更新 src/protocols/
3. frontend/backend 必须同步实现变更
4. tester 必须更新对应测试用例
