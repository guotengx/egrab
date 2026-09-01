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
| cdp | src-tauri/src/cdp/ | CDP连接管理、WebSocket通信、页面导航 | planner | ✅ done | L5 cdp-manager | ✅ unit + serde (12 tests) |
| scraper | src-tauri/src/scraper/ | 抓取引擎核心，协调解析器和下载器；wait/expand/scroll 参数从规则包读取 | planner | ✅ done | L5 scraper-engine | ✅ unit |
| parser/rules | src-tauri/src/parser/rules.rs | **规则驱动的通用解析器**（v0.2.0 起替代所有平台专属解析器） | planner | ✅ done | L5 parser | ✅ unit (9 tests) |
| rules（规则包） | src-tauri/rules/ | 外置抓取规则：rules.json + 各平台 extract/expand JS，随二进制内嵌并释放到磁盘 | planner | ✅ done | rules/README.md | ✅ 内嵌规则校验 + JS 离线回归 |
| ~~parser/taobao~~ | ~~src-tauri/src/parser/taobao.rs~~ | 已于 v0.2.0 删除（558 行），逻辑迁移至 `rules/taobao.extract.js` | - | ❌ removed | - | - |
| ~~parser/jd~~ | ~~src-tauri/src/parser/jd.rs~~ | 已于 v0.2.0 删除（727 行），逻辑迁移至 `rules/jd.extract.js` | - | ❌ removed | - | - |
| downloader | src-tauri/src/downloader/ | 图片批量下载器（并发控制、重试） | planner | ✅ done | L5 downloader | ✅ unit |
| storage | src-tauri/src/storage/ | 存档引擎（SQLite + JSON + 文件系统） | planner | ✅ done | L5 storage | ✅ unit |
| commands | src-tauri/src/commands/ | Tauri IPC命令定义（前端可调用的接口） | planner | ✅ done | L5 ipc-commands | ✅ unit + serde (6 tests) |
| config | src-tauri/src/config/ | 应用配置管理 | planner | ✅ done | L5 config | ✅ unit |
| pages | src/pages/ | 页面组件（Home, Progress, Archive, Settings） | planner | ✅ done | - | ✅ tsc |
| components | src/components/ | 可复用UI组件 | planner | ✅ done | - | ✅ tsc |
| stores | src/stores/ | Svelte状态管理（连接状态、任务状态等） | planner | ✅ done | - | ✅ tsc |
| resize | src-tauri/src/resize/ | 图片等比缩放（输出到 proportioned/ 子目录） | planner | ✅ done | L5 data-models | ✅ tsc |
| services | src/services/ | Tauri IPC调用封装 | planner | ✅ done | L5 ipc-commands | ✅ tsc |

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

### 遗留风险清单（未决，非本期修复范围）

- **R-1 `identifier` 双轨隐患（2026-09-01 发现，口头承诺记录但此前漏记，现补录）**：
  `tauri.conf.json` 的 identifier 是 `com.egrab.desktop`，而 Rust 代码数据目录全部硬编码为
  `com.egrab.app`（规则包、index.db、CDP profile）。目前两边各自自洽所以正常跑，但属于定时炸弹——
  一旦改用 Tauri 官方 path API 就会指向另一个目录；若改 identifier 会变更 MSI UpgradeCode 导致
  新版装不上（并存两个）；若改 Rust 路径会让用户已抓数据和登录态"凭空消失"。
  **需要单独设计一次数据迁移才能动，本期不动。**
- **R-2 Windows 代码签名**：360 误报（2026-06-11 记录）的长期方案是购买 Authenticode 证书，未实施。
- **R-3 v0.2.1 待真机验证**：CI 构建结果 + 用户实测 counts 段（预期 gallery=5 / detail=12），
  以及 %LOCALAPPDATA%\EGrab 错架构残留需用户手动卸载。

---

*最后更新: 2026-09-01 by planner (TD-011 用户实测修复轮 + R-1~R-3 遗留风险补录；TD-010 产物表已按 v0.2.1 修正)*
