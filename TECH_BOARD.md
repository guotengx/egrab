# EGrab - 技术看板

> 由 planner 维护。记录前后端接口对接进度、模块开发状态和技术决策。
> **当前模式**：单 Agent 模式（planner 全能开发者独立完成所有工作）。

---

## Phase 5 开发计划

### 开发批次与优先级

| 批次 | 范围 | 目标 | 依赖 | 状态 |
|------|------|------|------|------|
| **P5-1** | 基础 IPC 通信 | get_config + set_config 前后端联调验证 | models 已就绪 | ✅ 完成 |
| **P5-2** | CDP 连接管理 | cdp_connect/cdp_disconnect/cdp_status/cdp_list_tabs | P5-1 | ✅ 完成 |
| **P5-3** | 配置 + 前端骨架 | config 模块完整 + 前端 App/Home/Settings 页面 | P5-1 | ✅ 完成 |
| **P5-4** | 存储引擎 | SQLite + JSON + 文件系统 + get_task_history/get_task_detail/open_folder | P5-1 | ✅ 完成 |
| **P5-5** | 抓取核心流程 | scraper engine + start_scrape/cancel_scrape + 进度/完成事件 | P5-2, P5-4 | ✅ 完成 |
| **P5-6** | 平台解析器 | 淘宝/京东解析器 | P5-5 | ✅ 完成 |
| **P5-7** | 图片下载器 | 并发下载 + 重试 | P5-5 | ✅ 完成 |
| **P5-8** | 前端完整 UI | Progress/Archive 页面 + 全部组件 + stores | P5-3, P5-5 | ✅ 完成 |

---

## 模块开发状态

| 模块 | 路径 | 职责 | 负责人 | 状态 | 接口定义 | 测试覆盖 |
|------|------|------|--------|------|---------|---------|
| models | src-tauri/src/models/ | 全局数据模型定义（与PRD 3.1.2对齐） | planner | ✅ done | L5 protocols | ✅ unit + serde (25 tests) |
| cdp | src-tauri/src/cdp/ | CDP连接管理、WebSocket通信、页面导航；v0.2.2 新增 `bring_to_front`（Page.bringToFront） | planner | ✅ done | L5 cdp-manager | ✅ unit + serde (12 tests) |
| scraper | src-tauri/src/scraper/ | 抓取引擎核心，协调解析器和下载器；wait/expand/scroll/ready 参数从规则包读取；v0.2.2 起导航后切前台、滚动后轮询 ready_js，raw.json 写入 driver/rules 诊断段 | planner | ✅ done | L5 scraper-engine | ✅ unit (7 tests) |
| parser/rules | src-tauri/src/parser/rules.rs | **规则驱动的通用解析器**（v0.2.0 起替代所有平台专属解析器）；v0.2.2 新增 ready_js / ready_js_file / ready_timeout_ms 与 `find_rule_with_source` | planner | ✅ done | L5 parser | ✅ unit (11 tests) |
| parser/utils | src-tauri/src/parser/utils.rs | 图片 URL 清洗（淘宝/京东）、价格解析；v0.2.2 修正 `.png.avif` 清洗 | planner | ✅ done | - | ✅ unit (19 tests) |
| rules（规则包） | src-tauri/rules/ | 外置抓取规则：rules.json + 各平台 extract/expand/ready JS，随二进制内嵌并释放到磁盘；当前 **v6**（京东新增 jd.ready.js） | planner | ✅ done | rules/README.md | ✅ 内嵌规则校验 + `tests/rules/` 真实页面离线回归 37/37 |
| ~~parser/taobao~~ | ~~src-tauri/src/parser/taobao.rs~~ | 已于 v0.2.0 删除（558 行），逻辑迁移至 `rules/taobao.extract.js` | - | ❌ removed | - | - |
| ~~parser/jd~~ | ~~src-tauri/src/parser/jd.rs~~ | 已于 v0.2.0 删除（727 行），逻辑迁移至 `rules/jd.extract.js` | - | ❌ removed | - | - |
| downloader | src-tauri/src/downloader/ | 图片批量下载器（并发控制、重试）；v0.2.2 起按文件头识别真实格式决定扩展名 | planner | ✅ done | L5 downloader | ✅ unit (15 tests) |
| storage | src-tauri/src/storage/ | 存档引擎（SQLite + JSON + 文件系统） | planner | ✅ done | L5 storage | ✅ unit |
| commands | src-tauri/src/commands/ | Tauri IPC命令定义（前端可调用的接口） | planner | ✅ done | L5 ipc-commands | ✅ unit + serde (6 tests) |
| config | src-tauri/src/config/ | 应用配置管理 | planner | ✅ done | L5 config | ✅ unit |
| pages | src/pages/ | 页面组件（Home, Progress, Archive, Settings） | planner | ✅ done | - | ✅ tsc |
| components | src/components/ | 可复用UI组件 | planner | ✅ done | - | ✅ tsc |
| stores | src/stores/ | Svelte状态管理（连接状态、任务状态等） | planner | ✅ done | - | ✅ tsc |
| resize | src-tauri/src/resize/ | 图片等比缩放（输出到 proportioned/ 子目录）；v0.2.2 起按内容解码、JPEG 目标先转 RGB、失败复制原图 | planner | ✅ done | L5 data-models | ✅ unit (2 tests) |
| 诊断日志 | src-tauri/src/lib.rs | v0.2.2 起日志同时写 stderr 与 `<app_data>/logs/egrab.log`（>5MB 启动时轮转） | planner | ✅ done | - | - |
| services | src/services/ | Tauri IPC调用封装 | planner | ✅ done | L5 ipc-commands | ✅ tsc |

> **注（2026-10-29）**：v0.2.2 新增的 8 个 Rust 单测从未执行过：
> - rules 2 个、utils 1 个、downloader 3 个、resize 2 个；
> - 原因：本地没有 Rust 工具链，CI 只跑构建、不跑 `cargo test`（R-6）。
>
> 表中的测试数量只表示「已编写」，不表示「已通过」。

---

## 接口联调状态

| IPC命令 | 后端实现 | 前端调用 | 联调通过 |
|---------|---------|---------|---------|
| cdp_connect | ✅ done | ✅ done | ✅ 类型对齐 |
| cdp_disconnect | ✅ done | ✅ done | ✅ 类型对齐 |
| cdp_status | ✅ done | ✅ done | ✅ 类型对齐 |
| cdp_list_tabs | ✅ done | ✅ done | ✅ 类型对齐 |
| start_scrape | ✅ done | ✅ done | ✅ 类型对齐 |
| cancel_scrape | ✅ done | ✅ done | ✅ 类型对齐 |
| get_task_history | ✅ done | ✅ done | ✅ 类型对齐 |
| get_task_detail | ✅ done | ✅ done | ✅ 类型对齐 |
| open_folder | ✅ done | ✅ done | ✅ 类型对齐 |
| get_config | ✅ done | ✅ done | ✅ 类型对齐 |
| set_config | ✅ done | ✅ done | ✅ 类型对齐 |
| cdp_auto_connect | ✅ done | ✅ done | ✅ 类型对齐（v0.1.12 起用于 Edge 兼容） |
| cdp_navigate | ✅ done | ✅ done | ✅ 类型对齐 |
| delete_task | ✅ done | ✅ done | ✅ 类型对齐 |
| get_cover_image | ✅ done | ✅ done | ✅ 类型对齐 |
| resize_images | ✅ done | ✅ done | ✅ 类型对齐 |
| get_rules_info | ✅ done (v0.2.0) | ✅ done | ✅ 类型对齐 |
| reload_rules | ✅ done (v0.2.0) | ✅ done | ✅ 类型对齐 |
| open_rules_folder | ✅ done (v0.2.0) | ✅ done | ✅ 类型对齐 |
| dump_page_snapshot | ✅ done (v0.2.0) | ✅ done | ✅ 类型对齐 |

> **注**：v0.2.0 新增的 4 个规则/诊断命令属于运维辅助接口，未写入 L2 ARCHITECTURE 5.1 的核心 IPC 表，
> 不影响 AGENTS.md §2.2 的命名一致性铁律（该铁律约束的是 12 个核心业务命令）。
> 实际上 cdp_navigate / delete_task / get_cover_image / resize_images 也不在 L2 表里；
> L4/L5 协议层同样缺失。完整漂移清单见 STATUS C-10（2026-10-29 补充）。

---

## Phase 6 测试计划

### 测试批次与优先级

| 批次 | 范围 | 目标 | 测试文件 | 状态 |
|------|------|------|---------|------|
| **P6-1** | 数据模型序列化 + CDP 状态 + 命令验证 | models/cdp/commands 核心类型 serde round-trip | models_serde_test.rs (25), cdp_state_test.rs (12), commands_test.rs (6) | ✅ 完成 (43/43 通过) |
| **P6-2** | 存储引擎集成测试 | SQLite CRUD + 文件系统存档 + 去重检测 | storage_integration_test.rs (6) | ✅ 完成 (6/6 通过) |
| **P6-3** | 抓取引擎端到端测试 | scraper engine 协调流程 + 事件发射 | scraper_test.rs (5) | ✅ 完成 (5/5 通过) |

### P6-1 测试详情

**总测试数：43（通过 43，失败 0）**

| 测试文件 | 测试数 | 覆盖类型 |
|---------|-------|---------|
| models_serde_test.rs | 25 | ProductData, ImageRef, SkuItem, PriceRange, ShopInfo, Description, SpecItem, ConnectionState, AppConfig, BrowserLaunchCommand, MetaJsonDocument, RawJsonDocument, TaskFilter 序列化/反序列化 round-trip |
| cdp_state_test.rs | 12 | ConnectionState 5 变体序列化+反序列化+round-trip, ConnectionInfo, TabInfo, CdpEndpoint |
| commands_test.rs | 6 | URL 空字符串验证, ErrorCode+IpcError 组合, TaskStatus 序列化, ScrapeStep 序列化 |

---

## Spike 验证规划

> Phase 5 前置技术验证任务，用于消除关键技术风险。每项 Spike 产出最小可运行验证代码，不进入主分支。

### Spike 1：chromiumoxide + tokio 运行时验证

| 项目 | 内容 |
|------|------|
| **目标** | 验证 chromiumoxide 与 Tauri async runtime 配合方式 |
| **负责人** | planner |
| **验证内容** | 1. Cargo.toml 中 chromiumoxide 的 features 配置（需要 `tokio-runtime`，禁用默认 features）<br>2. 连接 `127.0.0.1:9222` → `navigate("about:blank")` → `evaluate("1+1")` → 断言返回 `2`<br>3. handler future 必须 spawn 后台任务驱动 |
| **验收标准** | 最小可运行的 CDP 连接 + evaluate 示例 |
| **状态** | pending |

### Spike 2：淘宝解析兜底链路验证

| 项目 | 内容 |
|------|------|
| **目标** | 验证淘宝商品页多来源解析策略 |
| **负责人** | planner |
| **验证内容** | 1. 准备 3 个不同年代的淘宝商品 URL（老 PC 站/H5/天猫旗舰店）<br>2. 验证 `g_config`、`__INITIAL_DATA__`、`window.__data__`、SSR JSON 等多来源解析<br>3. 断言三套 parser 路径都能产出非空 ProductData |
| **验收标准** | 至少 2 个来源解析成功，失败时正确记录 raw_data 并标记 partial |
| **状态** | ⛔ 已由 TD-009 取代 —— 多来源兜底逻辑已下沉到规则脚本 `taobao.extract.js`（ICE 路由遍历兜底 + DOM 兜底），不再需要独立 Spike |

### Spike 3：Windows 打包体积测试

| 项目 | 内容 |
|------|------|
| **目标** | 验证 Windows 打包体积是否 < 15MB |
| **负责人** | planner |
| **验证内容** | 1. `cargo build --release` 后测量体积<br>2. 如果超标，尝试优化方案：<br>　　- reqwest 仅启用 `rustls-tls`<br>　　- tokio 选择最小 feature 子集<br>　　- `[profile.release] opt-level = "z", lto = true, codegen-units = 1, strip = true` |
| **验收标准** | Windows .msi/.exe < 15MB，或给出明确优化方案 |
| **状态** | pending |

---

## 技术决策日志

### TD-001: Phase 5 开发策略 - 增量联调
- **日期**: 2026-05-09
- **决策**: 采用"最小 IPC 联调先行"策略，先实现 get_config/set_config 验证前后端通信链路，再逐步扩展到 CDP、抓取、存储等复杂模块
- **原因**: 前端骨架尚未建立（无 App.svelte/main.ts），需要先建立前端基础结构；后端 commands 模块尚未注册到 lib.rs；通过最简单的 IPC 命令验证全链路可大幅降低后续集成风险

### TD-002: chromiumoxide API 适配
- **日期**: 2026-05-10
- **决策**: 修复 CDP manager 中 chromiumoxide API 调用错误
- **变更**:
  1. `page.page_id()` → `page.target_id()` (chromiumoxide 0.4 API)
  2. `page.title()` → `page.get_title()` (返回 `Option<String>`)
  3. `page.url()` 返回 `Option<String>` 而非 `String`
  4. `page.evaluate()` 返回 `EvaluationResult`，需 `.into_value()` 转换为 `serde_json::Value`
  5. 添加 `use tauri::Emitter;` 以启用 `AppHandle::emit()` 方法
  6. 移除未使用的 `BrowserConfig` import
- **原因**: chromiumoxide 0.4 API 与初始代码假设不一致，编译期发现并修复

### TD-003: StorageEngine 使用 tokio::sync::Mutex
- **日期**: 2026-05-10
- **决策**: 将 StorageEngine 的 Mutex 从 `std::sync::Mutex` 改为 `tokio::sync::Mutex`
- **原因**: ScraperEngine 的 `start_scrape` 方法需要跨 `.await` 点持有 StorageEngine 的锁（CDP 操作期间需要更新任务状态）。`std::sync::MutexGuard` 不是 `Send`，导致 Tauri command handler 的 future 不满足 `Send` 约束。改用 `tokio::sync::Mutex` 后，`MutexGuard` 是 `Send`，解决了跨 await 持锁问题。
- **影响**: task_commands.rs 和 scraper/engine.rs 中的 `storage.lock()` 调用从 `.lock().map_err(...)` 改为 `.lock().await`

### TD-004: ScraperEngine 不作为 Tauri managed state
- **日期**: 2026-05-10
- **决策**: ScraperEngine 不注册为 Tauri managed state，而是在 IPC command handler 中通过 `AppHandle` 参数临时创建
- **原因**: ScraperEngine 只包含一个 `AppHandle`（轻量级），不需要持久化状态。直接在 command handler 中接收 `tauri::AppHandle` 参数避免了额外的 Mutex 包装和 Send 约束问题。

### TD-005: CdpPageHandle 使用 raw pointer 桥接
- **日期**: 2026-05-10
- **决策**: CdpPageHandle 使用 `*const CdpManager` raw pointer 实现 PageHandle trait
- **原因**: CdpManager 通过 `tauri::State` 访问，返回 `&CdpManager` 引用。PageHandle trait 要求 `Send + Sync`，而引用的生命周期无法满足 'static 约束。由于 CdpManager 由 Tauri 管理且生命周期与应用相同，CdpPageHandle 仅在 start_scrape 的 async scope 内使用，raw pointer 方案是安全的。
- **安全性**: 添加了 `unsafe impl Send/Sync for CdpPageHandle`，并在所有解引用处添加了安全注释。

### TD-006: rusqlite 0.30 API 修复
- **日期**: 2026-05-10
- **决策**: 修复 storage/database.rs 中的 rusqlite API 调用
- **变更**:
  1. `query_row().optional()` → `query_row().ok()`（rusqlite 0.30 中 `optional()` 方法不在 `Result` 上）
  2. `raw_query().and_then()` → `query_map()`（简化动态参数查询）
  3. `execute_dynamic_update()` → 使用 `Box<dyn ToSql>` 参数列表直接调用 `Connection::execute()`
  4. 移除未使用的 `ParamValue` 枚举
- **原因**: 原代码假设的 rusqlite API 与 0.30 版本不匹配，编译期发现并修复

### TD-007: 新增 cdp_auto_connect IPC 命令
- **日期**: 2026-05-10
- **决策**: 新增 `cdp_auto_connect` IPC 命令，支持应用启动后自动检测浏览器、自动启动 CDP、自动连接
- **变更**:
  1. `src/protocols/ipc-commands.ts`：新增 `CdpAutoConnectCommand` 接口（name: 'cdp_auto_connect', params: 无, returns: ConnectionInfo），加入 `IpcCommand` 联合类型
  2. `src/protocols/data-models.ts`：`ErrorCode` 新增 `NO_BROWSER_FOUND`（未检测到浏览器）和 `CDP_LAUNCH_TIMEOUT`（浏览器启动超时）
- **原因**: 用户需求——EGrab 打开后自动检测浏览器、自动启动 CDP、自动连接
- **影响范围**:
  - 后端需新增 `cdp_auto_connect` Tauri command 实现（cdp 模块 + commands 模块）
  - 前端需在 services/ipc.ts 新增调用封装，stores/connection.ts 需支持自动连接流程
  - L4 `docs/protocols/ipc-commands.md` 需同步更新（需 pre agent 处理）：命令列表新增、约束第 128 行"不新增 IPC 命令"需改为允许 `cdp_auto_connect`、第 126 行命令名列表需新增
  - L2 `docs/ARCHITECTURE.md` 第 5.1 节 IPC 命令表需新增
  - `AGENTS.md` 命名一致性铁律中的 IPC 命令名列表需新增

---

## Phase 5+6 全面检查报告（2026-05-10）

### 检查范围
Phase 5（前后端开发）和 Phase 6（测试联调）的代码质量、接口一致性、测试覆盖率和一致性审计。

### 检查结果

#### 1. 代码质量

**后端 Rust（src-tauri/src/）**：✅ 良好
- 所有 11 个 IPC 命令已注册
- ProductData 九字段与 PRD 3.1.2 完全对齐
- ErrorCode 使用 SCREAMING_SNAKE_CASE serde（P0-1 已修复验证）
- IpcError.code 使用 ErrorCode 枚举（P0-4 已修复验证）
- 所有模块有 mod.rs 入口和顶部注释
- 错误处理统一使用 IpcError
- 无无注释 unwrap()

**前端 TypeScript/Svelte（src/）**：✅ 良好
- 所有 11 个 IPC 命令在 services/ipc.ts 中封装
- 所有 4 个事件在 services/events.ts 中封装
- 3 个 stores 使用 Svelte 5 runes
- 4 页面 + 4 组件，类型从 protocols 导入
- Raycast 暗黑主题设计 Token 已应用
- ss03 字体特性在 App.svelte 中内联应用
- tsc --noEmit 零错误

**轻微问题**：
- ⚠️ PriceRange.currency：TypeScript 为 `'CNY'` 字面类型，Rust 为 `String`（功能等价，类型严格度差异）
- ⚠️ chrono_now_str()/days_to_date() 在 storage/mod.rs 和 filesystem.rs 中重复
- ⚠️ services/ipc.ts 有过时 TODO 注释
- ⚠️ tailwind.config.js 缺少部分 Raycast Token（accent 颜色等），组件中已使用但未在配置中定义

#### 2. 测试验证

**全量测试**：✅ 131/131 通过
- cargo test: 131 passed, 0 failed, EXIT_CODE=0
- tsc --noEmit: 0 errors, EXIT_CODE=0

**测试覆盖**：
| 测试类别 | 测试文件 | 测试数 | 覆盖范围 |
|---------|---------|-------|---------|
| 模型序列化 | models_serde_test.rs | 25 | ProductData/ImageRef/SkuItem/PriceRange 等 serde round-trip |
| CDP 状态 | cdp_state_test.rs | 12 | ConnectionState 5 变体/ConnectionInfo/TabInfo/CdpEndpoint |
| 命令验证 | commands_test.rs | 6 | URL 验证/ErrorCode+IpcError/TaskStatus/ScrapeStep |
| 存储集成 | storage_integration_test.rs | 6 | SQLite CRUD/去重/force 覆盖/历史查询 |
| 抓取引擎 | scraper_test.rs | 5 | URL 验证/item_id 提取/事件结构 |
| 内联单元 | 各模块 #[cfg(test)] | 77 | 数据库操作/文件系统/URL 清洗/解析器/下载器 |

**未覆盖**（预期）：
- E2E 测试需要真实 CDP 浏览器连接
- 前端组件测试未设置（Svelte 组件测试框架未配置）

#### 3. 一致性审计（reviewer）

**总体评级**：✅ 通过

| 审计维度 | 结果 | 详情 |
|---------|------|------|
| ProductData 九字段 | ✅ 通过 | L1→L2→L4→L5→L6 全链路零差异 |
| IPC 命令名 | ✅ 通过 | 11 个命令全链路一致 |
| 事件名 | ✅ 通过 | 4 个事件全链路一致 |
| TS/Rust 类型对齐 | ✅ 通过 | serde rename_all 保证 JSON 字段名一致 |
| start_scrape 签名 | ✅ 通过 | url + force? 参数一致 |
| ErrorCode 枚举 | ✅ 通过 | SCREAMING_SNAKE_CASE 序列化对齐 |
| ConnectionState 格式 | ✅ 通过 | tagged union PascalCase 一致 |
| 数据流分层 | ✅ 通过 | 组件→stores→services→IPC 无越层 |
| 权限一致性 | ✅ 通过 | opencode.json 无 deny 配置 |

#### 4. 待修复项状态（D-1 到 D-9）

| # | 问题 | 优先级 | 当前状态 | Phase 7 影响 |
|---|------|--------|---------|-------------|
| D-1 | IpcResult vs Tauri Result 语义双轨 | P1 | 未修复 | 不阻塞（功能正确） |
| D-2 | open_folder 安全策略与 Tauri 2 capabilities | P1 | 未修复 | 不阻塞（路径校验已实现） |
| D-3 | force=true 重抓事务语义 | P1 | 未修复 | 不阻塞（基本 force 覆盖已工作） |
| D-4 | 淘宝/京东解析兜底链路 | P1 | 未修复 | 不阻塞（需 Spike 2 验证） |
| D-5 | STATUS.md/TECH_BOARD 表述过早 | P1 | 未修复 | 不阻塞（文档问题） |
| D-6 | tailwind.config.js 缺少 ss03 | P2 | 未修复 | 不阻塞（已内联应用） |
| D-7 | tauri.conf.json CSP 与 bundle | P2 | 未修复 | Phase 7 需处理 |
| D-8 | src/protocols/README.md 过时 | P2 | 未修复 | 不阻塞（文档问题） |
| D-9 | AGENTS.md §5 reviewer 读取要求 | P2 | 未修复 | 不阻塞（文档问题） |

### 结论
Phase 5 和 Phase 6 全面检查通过。代码质量良好，接口一致性全链路零差异，131 个测试全部通过，reviewer 一致性审计通过。D-1 到 D-9 均为已记录的 P1/P2 延后项，不阻塞 Phase 7 打包交付。建议 Phase 7 优先处理 D-7（tauri.conf.json CSP 配置）。

### TD-008: 图片等比缩放架构重构 — 输出到 proportioned/ 子目录

- **日期**: 2026-05-15
- **决策**: 将 resize 模块从"原地覆盖原图"改为"输出到 `proportioned/` 子目录，保持原图不动"，并在 scraper engine 抓取完成后自动调用
- **变更**:
  1. `src-tauri/src/resize/mod.rs`：`resize_images_in_folder` 函数签名不变，但内部逻辑从 `resized.save(path)` 改为 `resized.save(proportioned_path)`，输出到 `{folder}/proportioned/{cover,gallery,detail,sku}/` 子目录
  2. `src-tauri/src/scraper/engine.rs`：`run_scrape` 方法在 Step 6（保存结果）之后、emit complete 之前，自动调用 `resize_images_in_folder` 对当前任务文件夹执行等比缩放
  3. `src/protocols/data-models.ts`：新增 `ResizeResult` 和 `ResizeDetail` 类型定义，供前端使用
  4. `src/protocols/ipc-commands.ts`：`ResizeImagesCommand.returns` 从内联对象改为引用 `ResizeResult` 类型
  5. `src/services/ipc.ts`：`resizeImages` 返回类型从内联改为 `ResizeResult`
  6. `src/pages/Archive.svelte`：已有"压缩图片"按钮，需更新文案为"等比缩放"
- **原因**: 用户需求——抓取时自动处理 + 前端手动触发 + 原图不动 + 只等比缩放不压缩
- **目录结构变更**:
  ```
  taobao_12345678_20260505T143022/
  ├── meta.json
  ├── raw.json
  ├── cover/             # 原图（不动）
  ├── gallery/           # 原图（不动）
  ├── detail/            # 原图（不动）
  ├── sku/               # 原图（不动）
  └── proportioned/      # 新增：等比缩放输出
      ├── cover/
      ├── gallery/
      ├── detail/
      └── sku/
  ```
- **影响范围**:
  - **backend**：`resize/mod.rs`（核心逻辑改为输出到 proportioned/）、`scraper/engine.rs`（新增自动调用）、`commands/resize_commands.rs`（无需改动，已通过 folder_path 调用）
  - **frontend**：`services/ipc.ts`（返回类型对齐 ResizeResult）、`pages/Archive.svelte`（按钮文案更新）
  - **architect**：`src/protocols/data-models.ts`（新增类型）、`src/protocols/ipc-commands.ts`（引用新类型）、`src/protocols/index.ts`（导出新类型）
  - **tester**：需新增 resize 输出到 proportioned/ 的测试用例

---

### TD-009: 抓取规则外置引擎 —— 解析逻辑从二进制中剥离（v0.2.0）

- **日期**: 2026-08-29
- **背景**: 京东主图、天猫价格/规格/SKU 同时抓不到。根因排查结论：
  1. **京东**：`pageConfig.product` 已被平台清空（仅剩 `chooseLOCShop`/`colorApiDomain`），
     DOM 兜底用的 `._gallery_116km_1` 是 CSS-Module **构建哈希类名**，京东每次发版哈希都变 → `GALLERY_EMPTY`
  2. **天猫**：数据源仍在 `__ICE_APP_CONTEXT__`，但取值路径全部错位：
     - 价格实际在 `skuCore.sku2info[skuId].price.priceMoney`（单位分），旧代码读 `sku2info/0.priceText`（该键不存在）
     - 规格实际在 `plusViewVO.industryParamVO.basicParamList`，旧代码找已废弃的 `.attributes-list li`
     - SKU 实际在 `skuBase.props` + `skuBase.skus` + `skuCore.sku2info`，旧代码找已废弃的 `g_config.idata.sku`
- **核心结论**: 平台改版是常态而非异常，**把易变的解析规则编译进二进制是架构错误**。
  每次适配都要走「改 Rust → CI 编译 10 分钟 → 重新分发安装包 → 全员重装」，成本与收益严重失衡。
- **决策**: 建立外置规则包引擎，解析规则以可编辑文件形式存放于磁盘，抓取时动态加载。

- **架构**:
  ```
  src-tauri/rules/                    随二进制 include_str! 内嵌（永久兜底）
    ├── rules.json                    平台匹配 / item_id 提取 / 就绪判定 / 滚动参数
    ├── {platform}.extract.js         数据提取脚本（返回 ProductData 形状的 JSON）
    ├── {platform}.expand.js          详情区展开脚本（滚动前执行）
    └── README.md                     排查手册 + 抗改版写法

           ↓ 首次启动释放 / 版本升级时备份覆盖

  <app_data>/com.egrab.app/rules/     用户可直接编辑，改完保存即生效
    └── snapshots/                    dump_page_snapshot 输出
  ```

- **关键设计**:
  | 设计点 | 做法 | 目的 |
  |--------|------|------|
  | 加载优先级 | 磁盘 > 内嵌 | 用户改动优先，同时保证永远有可用规则 |
  | 容错 | 磁盘 JSON 解析失败 → 自动回退内嵌 + 记录 error | 改坏规则文件不会让程序变砖 |
  | 无缓存 | 每次抓取重新读盘 | 编辑后立即生效，不需重启 |
  | 版本升级 | 内嵌 version > 磁盘 version 时备份为 `*.bak` 后覆盖 | 既能推送修复，又不静默丢用户改动 |
  | 安全 | `*_js_file` 拒绝含 `/` `\` `..` 的文件名 | 防目录穿越 |
  | 扩展性 | 新增平台 = 改 rules.json + 加一个 JS 文件 | 零 Rust 改动即可支持 1688/拼多多等 |

- **Rust 侧职责收缩为**：执行 JS → 反序列化 → 图片 URL 清洗（`image_cleaner: taobao/jd/none`）→ 映射为 `ProductData`。
  **不再含任何平台专属逻辑**，删除 `parser/jd.rs`(727 行) 与 `parser/taobao.rs`(558 行)。

- **抗改版编码规范**（已写入 `rules/README.md`）：
  ```js
  // ❌ 平台发版即失效
  document.querySelectorAll('._gallery_116km_1 .image-carousel-track img')
  // ✅ 子串匹配，哈希变了照样命中
  document.querySelectorAll('[class*="gallery"] img, [class*="carousel"] img')
  ```
  同理 SSR JSON 不写死路由名，`ice.loaderData.home` 取不到时遍历 `loaderData` 找含 `item` 的节点。

- **配套诊断能力**:
  - `dump_page_snapshot`：导出完整 DOM + 12 个候选全局变量 + 图片清单 + id/class 清单 → `snapshots/`
  - `raw.json` 新增 `counts` 段（gallery/detail_images/skus/specs/price_min/price_max），一眼定位失效字段
  - 新增 `PRICE_MISSING`、`SPECS_EMPTY`、`EXTRACT_JS_RETURNED_NULL` 三个降级告警码
  - 设置页新增：打开规则目录 / 校验规则 / 导出页面快照

- **验证方式**: 本机无 Rust 工具链，采用两级离线验证：
  1. Node 校验 `rules.json` 合法性 + 4 个 JS 文件语法
  2. 用用户提供的**真实天猫 raw.json** 构造 fixture，stub DOM 后跑 `taobao.extract.js`，
     8 条回归断言全通过（price 168.06~198、specs 7 条、sku price=198 stock=200 等）
  3. `npx tsc --noEmit` 零错误；Rust 首次编译由 CI 承担

- **遗留风险**: 京东选择器缺少真实 DOM 快照校准，属最佳猜测。若 `counts.gallery` 仍为 0，
  经 `dump_page_snapshot` 取快照后修改 `jd.extract.js` 即可，**该轮修复不需要重新编译**。

---

### TD-010: 构建矩阵扩展到双平台双架构（v0.2.0）

- **日期**: 2026-08-29
- **决策**: `.github/workflows/build.yml` 扩展为 4 个构建目标

| 平台 | 架构 | target triple | 产物 | 稳定性 |
|------|------|---------------|------|--------|
| macOS | aarch64 | `aarch64-apple-darwin` | `.dmg` | 稳定 |
| macOS | x86_64 | `x86_64-apple-darwin` | `.dmg` | 稳定 |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `.msi` + `.exe`(NSIS) | 稳定 |
| Windows | aarch64 | `aarch64-pc-windows-msvc` | `.exe`(NSIS) | 实验性 |

- **Windows ARM64 只出 NSIS 不出 MSI**：WiX v3 在 ARM64 上产 MSI 不稳定，NSIS 安装包在
  ARM64 Windows 上原生可用，是更可靠的路径。
- **该 job 标记 `continue-on-error: true`**，release job 条件改为
  `always() && needs.build-macos.result == 'success'`，确保实验性架构失败不阻断其余三个产物发布。
- macOS 改为 matrix 并行构建（此前是串行两次 build），DMG 文件名追加 `_aarch64` / `_x86_64` 后缀避免覆盖。
- 新增 cargo registry + target 缓存。
- **版本号统一对齐到 0.2.0**：`package.json` / `Cargo.toml` / `Cargo.lock` / `tauri.conf.json`
  此前一直停留在 0.1.0，导致 v0.1.1~v0.1.15 所有产物文件名都叫 `EGrab_0.1.0`。

> **TD-010 修正（v0.2.1，2026-09-01）**：上表中"Windows x86_64 产物 = MSI + NSIS"已废止。
> v0.2.0 用户实测发现双安装包互相冲突（MSI 装 `C:\Program Files\EGrab\`，NSIS currentUser 装
> `%LOCALAPPDATA%\EGrab\`，并存产生死快捷方式），且 %LOCALAPPDATA% 未签名 exe 易被杀软拦截。
> **自 v0.2.1 起 Windows x64 只出 MSI**，详见 TD-011。

---

### TD-011: 用户实测修复轮 —— JD 规则 v3/v4 + Windows 安装器修正（v0.2.1）

- **日期**: 2026-09-01
- **背景**: v0.2.0 发布后用户 Windows 真机实测，暴露两类问题——京东解析仍有噪声、Windows 安装包冲突。

#### 京东规则 v3（盲写内容过滤层，commit 13fc506）

- v0.2.0 的选择器"抗哈希但过宽"：实测 gallerySelectorHits=35（含图标）、cover 取到图标、
  detail 36 张混入推荐位、skus=0。修法是**加内容过滤而非收窄选择器**：
  | 机制 | 作用 |
  |------|------|
  | `isJdProductImage()` | 商品图必须在 `jfs/` 路径，排除装修/活动图路径 |
  | `passSizeGate()` | 拒收 <150px 小图，但放行 `sNxN_jfs` 尺寸标记的缩略图条（天然 54px，清洗后可取原图） |
  | `inExcludedZone()` | 排除 recommend/comment/rate/shop/banner/guess/hotsale/rank 区域 |
  | gallery 5 组分层 | 精确→宽泛逐组尝试，第一组有产出即停 |
  | 封面独立取大图 | `#spec-n1`/`mainImage`/`bigImg` 优先，退回 gallery[0] |
  | SKU 重写 | 覆盖 #choose-attrs 老版 + specification 新版，`data-value` 与内联文本双通道 |
- 自诊断字段：`gallerySample` / `coverUrl` / `detailSample` / `rejectedSample`（被拒样本+原因）/
  `galleryGroupUsed` / `skuRootCount` —— 保证下轮免快照定位。
- 实测残留：视频播放图标混入 gallery、2 张 imagetools 小图标混入 detail → 引出 v4。

#### 京东规则 v4（真实 HTML 校准，commit a12f306）

- 用户提供完整京东页面 HTML（item 10026681425538），按真实 DOM 定位 4 个 bug：
  1. **`imgzone` 误列噪声路径** —— 详情长图实测全在 `imgzone/jfs/`，v3 会全滤掉。移除。
  2. **`imgtools` 拼写错误** —— 京东实际是 `imagetools`。修正后视频播放图标被正确拦截。
  3. **`[class*="scoped"]` 详情容器过宽** —— 页面有 13 个 scoped 容器（店铺/标题/tab/参数/售后…），
     详情只是其一。改为按容器 ID 精确取（`#detail-main` / `#detail-top` / `#detail` / `related-layout-*`）。
  4. **gallery 播放图标** —— 缩略图条第一条 item 内叠 `img.thumbnails-play-icon`。
     改为优先 `.thumbnails img.image`（类名稳定），显式排除 play-icon，`#spec-img` 大图 + `jdKey` 去重。
- **真实结构基线（2026-09 存档）**：缩略图条 `.thumbnails .item img.image`（pcpubliccms/s228x228_jfs）、
  主图 `#spec-img`（pcpubliccms/s1440x1440_jfs）、详情 `#detail-main img`（imgzone/jfs/）、
  价格 `.product-price--main .product-price--value`、店铺 `.top-name`、
  SKU `.specification-item-sku`（`-image` s48x48_jfs + `-text`）、规格 `.attrs .item .label/.value`。
- 验证方式：按真实 HTML 还原 DOM stub，离线回归 **13/13 断言全过**
  （gallery 5 去重无图标 / 封面 s1440x1440 / 详情 12 张全 imgzone 且滤掉 2 张图标 /
  价格 87 不再抓错 187 / SKU 带 s48x48 图 / 规格 6 条）。
- 教训：**对抗性设计（抗哈希）只能保证方向对，不能保证细节对；没有真实页面样本时，
  路径黑名单类规则极易写错（imgzone 误杀、imagetools 拼写），容器级精确选择器 > 路径黑名单。**

#### Windows 安装器修正

- x64 只出 MSI（消除双包冲突）；`mainBinaryName: "EGrab"`（任务管理器/防火墙显示正确名称）；
  NSIS `perMachine` + `installerIcon` + 简体中文（ARM64 包也进 Program Files，规避杀软拦截）。
- 附带诊断结论：用户"ARM64 包打不开"是**架构装错**（x64 Windows 无法执行 ARM64 二进制，
  表现为空白图标+双击无反应），非杀软问题；快捷方式失灵根因是双安装包并存。

---

### TD-012: 运营实测修复轮 —— 京东规则 v6 + 抓取流程/图片格式修复（v0.2.2）

- **日期**: 2026-10-28
- **背景**: 运营把椰仙京东旗舰店的商品抓下来发布到拼多多，反馈 28 条问题。
  证据：9 个商品 10 份 raw.json、3 份浏览器渲染完成后保存的 MHTML、
  京东前端代码 `pc_item_components_3` 5.0.7（CDN 直接下载）、macOS Vision OCR。

| # | 根因 | 证据 | 修复 |
|---|------|------|------|
| 1 | 店铺编辑器插入的详情图 `<img class="shop-editor-zone-insert">` 自身命中 `[class*="shop"]`（`closest()` 包含元素自己） | 真实 DOM 跑规则：v4 详情 6/7/0 张，v5 9/8/17 张。v5（9-12）已修，但运营一直用 v0.2.1 内嵌 v4（无在线更新，手动拷贝未送达） | 规则 v6 随 v0.2.2 内嵌，启动自动覆盖磁盘 v4 |
| 2 | 「商品介绍」「规格参数」是两个独立的 IntersectionObserver 懒加载楼层（`enableInView`），抓取时未触发 | 4 个商品 `wrapperCount=0` 且 `specs=0`，主图/价格正常 | 引擎导航后 `Page.bringToFront`；滚动后轮询规则 `ready_js`（≤10s，脚本自行滚动触发楼层）；raw.json 新增 `driver`/`rules` 段，规则新增 `debug.page` |
| 3 | 京东对 `.png.avif` 地址返回 PNG，下载器按 URL 后缀存成 `.jpg`；缩放用 `image::open` 按扩展名解码，失败即静默跳过 | 8 个压缩包逐文件查文件头：PNG 内容的 `.jpg` 全部缺失于 `proportioned/`，真 JPEG 全在 | 下载按文件头存真实扩展名；缩放 `ImageReader::with_guessed_format()`，JPEG 目标先转 RGB，解码/编码失败复制原图 |
| 4 | 多款式：一个链接下多个 SKU，各有独立 skuId 与详情；DOM 上无 skuId | 有机水满香 3 个款式；前端代码确认点选款式时 `history.pushState` 改写地址栏 | 规则记录 `debug.skuSelected`；操作规避：点选款式后复制链接；一键抓全部款式待排期 |
| 5 | 拼多多包装标签图要求产品名/厂家/产地/地址/保质期/SC，京东页面上不存在合格标签图 | OCR 34 张详情 + 15 张主图：「产品信息」图只有品名/产地/保质期 | 非程序问题，需品牌方提供包装背面实拍 |

- **其他修正**：
  - `parser/utils.rs::clean_jd_image_url`：`.png.avif` 分支原先提前 return，跳过去尺寸前缀，
    meta.json 的 `original_url` 残留 `s48x48`（实际下载靠下载器第二道清洗兜住）。改为还原原格式并始终去前缀。
  - 规则：封面改为「选中款式图 > 第 1 张缩略图 > 主图区大图」（`#spec-img` 会随鼠标悬停切换）；
    店铺链接从 `a[href*="mall.jd.com/index-"]` 取；详情区块新增 `#detail-footer`；
    `imgSrc` 支持 `data-lazyload` 并识别占位 src。
  - 日志写入 `<app_data>/logs/egrab.log`（>5MB 启动时轮转）：release 包的失败任务首次有现场记录。
- **测试**：新增 `tests/rules/`（3 个真实页面 fixture + 最小 DOM 实现），`node tests/rules/jd.test.js` 37/37；
  Rust 新增单测（格式嗅探、扩展名替换、缩放兜底、ready 字段默认值、URL 清洗）。CI 只构建不跑 `cargo test`，见 R-6。
- **真实结构基线（2026-10 存档）**：
  - 详情数据来自 `pc_item_getWareGraphic`（需 h5st 签名，外部直连返回 `no access`，只能让页面自己渲染），
    区块全集：event-zone / related-layout-head / npcitem-notice / tx-video / detail-top / detail-main /
    detail-footer / related-layout-footer / industrial-introduction。椰仙店只有 `#detail-main`。
  - `left-tabs-item` 下规格楼层 `.attrs .item` 与详情楼层 `#detail-main` 各自懒加载；
    顶部「商品亮点」`.highlight-attrs .item` 首屏即有，不能当作规格楼层已加载的信号。
  - 店铺名 `div.top-name`（无 `<a>`），店铺链接 `a[href*="mall.jd.com/index-"]`；
    款式 `.specification-group > .specification-group-content > .specification-item-sku(--selected)`。
  - 详情图在渲染时就带真实 src：前端把 `graphicContent` 里的 `data-lazyload=` 整体替换成 `src=`，
    新版页面不存在逐图懒加载，只有楼层级懒加载。
  - 图片地址里的 `.jpg.avif` / `.png.avif` 是前端在 `localStorage.avif === "true"` 时追加的转码后缀，
    去掉即为原格式原图。
  - 资质接口 `pc_qualification_life_v2` 只返回商家营业执照链接，不含商品标签图，佐证了 C-8 的结论。
  - 匿名 curl 商品页只拿到约 35KB 的客户端渲染空壳；商品数据全部来自带 h5st 签名的接口。
  - 排查前先看商品页 HTML 引用的前端包版本号，本次是 5.0.7。版本变了，就要重新核对上面这些结构。
- **影响评估**：
  - 每次抓取 Chrome 窗口会被切到前台。
  - 最坏情况下每个商品多等约 10.8 秒（就绪超时 10 秒 + 800 毫秒），可能逼近 PRD 4.1「单商品 < 30 秒」。
  - 前端进度看门狗（`tasks.svelte.ts` 中 `PROGRESS_TIMEOUT_MS = 5_000`）只在收到第一个进度事件之前生效，
    不会被就绪等待误触发（已核实）。
- **拼多多侧参考**：开放平台 `pdd.goods.detail.get` 新增了 `package_label_gallery_list` 字段（包装标签图）。
  食品类必须单独上传包装标签图，生产商、保质期、许可证号等要清晰可见。以后如果做「导出到拼多多」可以参考。
- **更正**：v5 提交说明（b4a5f68）里写的根因「#detail-main 内嵌资质/标签子模块，class 含 shop/banner
  （shopQualification/cert-banner）」是当时没有真实 DOM 时的推断，**不成立**。真实原因是 `<img>` 自带的
  `shop-editor-zone-insert`。git 历史不改写，以本条为准。
- **证据与工具（可复现）**：
  - 证据位置见 STATUS「运营实测反馈台账」。
  - 京东前端包：`https://storage.360buyimg.com/retail-mall/pc_item_components_3/prod/5.0.7/index-legacy-*.js`
    （文件名从商品页 HTML 的 `<script>` 标签取）。
  - MHTML 解析：用 Python `email` 模块取 text/html 部分。**要手动按 UTF-8 解码**，因为该部分没声明 charset，
    默认会按 ASCII 解成乱码，导致款式名被去重成 1 个。
  - OCR：Swift + Vision `VNRecognizeTextRequest`（zh-Hans，accurate），macOS 自带，可直接读 AVIF。
  - **找用户要页面证据的标准做法**（按推荐顺序）：
    1. 浏览器里滚动到底、等商品介绍加载完，然后「另存为 → 网页，单个文件（.mhtml）」。保存的是渲染后的 DOM，图片原地址不变，还内嵌了已加载的图片。
    2. EGrab 设置页「导出页面快照」。注意它只读 EGrab 控制的那个标签页，要在那个标签页里手动滚完再点。
    3. DevTools 控制台执行 `copy(document.documentElement.outerHTML)`。
    
    **不要用**「查看网页源代码」：拿到的只是客户端渲染空壳 `<div id="root">`。**不要用**「另存为 网页，全部」：会把图片地址改写成本地文件。
  - 临时分析产物在 `$TMPDIR/opencode/egrab/{fb1028,mht}`（解压的 json、前端包、OCR 结果），会被系统清理；
    结论已写入本条和 STATUS 台账，原始证据在用户的 Downloads 目录。
- **教训**：
  1. **规则热更新的最后一公里是分发**：v5 早已修好，但没有送达机制，运营用 v4 跑了 6 周。
     规则修复要么随安装包内嵌发布，要么做在线更新。
  2. **`closest()` 会命中元素自身**：类名子串排除区要考虑 `<img>` 自带的 class。
  3. **抓到 ≠ 能用**：本轮最大的业务阻塞（拼多多标签图）不是抓取问题，用 OCR 在数据层面证实后才能剥离出去。

---

### 遗留风险清单（未决，非本期修复范围）

- **R-1 `identifier` 双轨隐患（2026-09-01 发现，口头承诺记录但此前漏记，现补录）**：
  `tauri.conf.json` 的 identifier 是 `com.egrab.desktop`，而 Rust 代码数据目录全部硬编码为
  `com.egrab.app`（规则包、index.db、CDP profile）。目前两边各自自洽所以正常跑，但属于定时炸弹——
  一旦改用 Tauri 官方 path API 就会指向另一个目录；若改 identifier 会变更 MSI UpgradeCode 导致
  新版装不上（并存两个）；若改 Rust 路径会让用户已抓数据和登录态"凭空消失"。
  **需要单独设计一次数据迁移才能动，本期不动。**
  补充（2026-10-29）：v0.2.2 新增的日志目录 `<app_data>/logs/` 同样位于硬编码的 `com.egrab.app` 下，
  将来迁移时也要一并处理。
- **R-2 Windows 代码签名**：360 误报（2026-06-11 记录）的长期方案是购买 Authenticode 证书，未实施。
- **R-3 v0.2.1 待真机验证**：✅ 已关闭（2026-10-28）—— v0.2.1 已被运营实际使用，实测问题由 TD-012 / v0.2.2 处理。
- **R-4 多款式一键抓取未实现**：同一链接下的其他款式需手动点选后复制链接逐个抓；
  DOM 上没有 skuId，实现需要引擎模拟点击款式并逐个提取（Rust + 前端 + 规则）。
- **R-5 规则没有在线更新**：规则修复只能随安装包内嵌下发或手动拷贝文件；v5 因此 6 周未送达运营电脑。
- **R-6 CI 不跑 `cargo test`**：build.yml 只执行 `tauri build`，Rust 单测从未在 CI 执行，可能悄悄失效。
  建议后续加一个 `cargo test` job（不阻断发布）。

以下 R-7 ~ R-14 是 2026-10 分析运营反馈时读代码发现、**本期未修复**的问题（2026-10-29 审计补录）：

- **R-7 前端平台白名单比规则包窄**：`src/components/UrlInput.svelte:20-24` 写死 3 条正则
  （item.taobao.com / detail.tmall.com / item.jd.com），规则包 `match_url` 支持的 item.m.jd.com、
  chaoshi.detail.tmall.com、detail.tmall.hk、item.tmall.com、world.taobao.com 都会被前端拦成「不支持的平台」。
  这削弱了「改规则即可支持新链接形态」的设计。
- **R-8 抓取标签页句柄不自愈**：`cdp/manager.rs` 的 `active_page` 只在连接/断开时重置，导航失败不重置。
  用户手动关掉 EGrab 打开的标签页后，后续抓取会一直失败，直到重新连接。
- **R-9 任务状态判定过粗**：`scraper/engine.rs` 的最终状态只看下载失败数，一张图都没提取到也算 success；
  导航失败和 `fail_task` 更新状态用的是 `try_lock`，锁被占用时任务会停在 Running。
- **R-10 PRD 3.1.3「下载失败降级为页面显示尺寸」未实现**：`downloader/image.rs` 下载原图失败后
  不会退回 `thumbnail_url`，图片直接丢失。
- **R-11 淘宝多款式冲突**：淘宝 `item_id` 只取 URL 的 `id`，忽略 `skuId`，同一商品的不同款式被当成同一商品。
  不勾强制重抓会报「已存在」；勾了会先删掉前一个款式的文件夹（`storage/mod.rs:126-139`）。
  主图和价格取的也是整个商品，不是选中款式。
- **R-12 京东 SKU 价格恒为 0**：`jd.extract.js` 的 skus 里 `price` 固定写 0；每个款式的 skuId 在 DOM 上拿不到（见 C-9）。
- **R-13 京东图片清洗有两份实现**：`parser/utils.rs` 与 `downloader/image.rs` 各有一份 `clean_jd_image_url`，
  后者没有 avif 处理，目前靠「解析时清洗一次 + 下载前再清洗一次」叠加兜底，属于技术债。
- **R-14 `jd.expand.js` 改样式范围过宽**：对所有 `[class*="scoped"]` 元素及其全部后代强改 height/overflow，
  可能影响页面布局；新版页面的详情图在渲染时已带真实 src，这个脚本的必要性需要重新评估。

---

*最后更新: 2026-10-29 by planner (记忆审计 + 二次复核：模块状态表同步 v0.2.2；TD-012 补更正、证据采集方法、结构细节、影响评估；R-1 补日志目录；新增 R-7~R-14)*
