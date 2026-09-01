# EGrab - 项目状态追踪

> 由 planner 维护，记录当前里程碑进度和任务分配状态。
> **当前模式**：单 Agent 模式（planner 全能开发者独立完成所有工作）。

---

## 当前里程碑：MVP-1（双平台基础抓取）

### 总体进度：100%

- Phase 0 初始化（PRD/ARCHITECTURE/AGENTS/opencode.json）：100% ✅
- Phase 1 制宪（pre agent 生成 contract + protocols）：100% ✅
- Phase 2 审计（qa + fallback + architect + tester + reviewer 五方审计）：100% ✅
- Phase 3 协议修复（pre 自检修复五方审计问题）：100% ✅（B-1~B-8 全部已修复，新增 4 份协议文档）
- Phase 4 架构接口预实现（src/protocols/ + models/）：100% ✅（architect 已生成 TypeScript 10文件 + Rust 5文件）
- Phase 4.5 全局一致性双检：100% ✅（pre 自检 + reviewer 独立审计，reviewer 定位已扩展为"一致性审计"）
- **Phase 4.6 模型更换后全局审核：100% ✅**（连通性测试 + reviewer/architect/qa/pre 四方审核，已修复协议层问题）
- **Phase 4.7 模型更换后全局审核（第二轮）：100% ✅**（11/11 agent 连通性测试 + 9 agent 一致性审核 + pre 自审）
- **Phase 4.8 P0 阻塞修复：100% ✅**（8 项 P0 全部修复并验证通过：P0-1~P0-8）
- **Phase 4.9 全局一致性审查：100% ✅**（pre 审查 10 份 contract + opencode.json + AGENTS.md，修复 npx --yes 白名单 + 权限说明）
- **Phase 5 前后端开发：100% ✅**（后端 11 IPC + 83 测试 ✅，前端 4 页面 + 4 组件 + 3 stores + 2 services 全部完成并验证 ✅）
- **Phase 6 测试联调：100% ✅**（P6-1 模型序列化 43/43 ✅，P6-2 存储引擎集成 6/6 ✅，P6-3 抓取引擎 E2E 5/5 ✅，全量 131 tests passed）
- **Phase 5+6 全面检查验证：100% ✅**（代码质量良好、131 测试全通过、一致性审计通过，Phase 7 可启动）
- **Phase 7 打包交付：100% ✅**（macOS DMG 5.1MB 构建成功，reviewer 运维审计通过）
- **Phase 7.1 JD 解析器修复 + 图片等比缩放重构：100% ✅**（JD detail_images 通用过滤 + resize 输出到 proportioned/ + scraper 自动调用 + frontend 按钮文案更新）
- **Phase 7.2 v0.1.3 本地构建：✅**（macOS ARM64 DMG 10MB ✅，< 15MB 限制）
- **Phase 7.2 v0.1.3 CI 构建：✅**（macOS aarch64+x86_64 + Windows x86_64）
- **Phase 7.3 JD 解析器普适性增强 (v0.1.5-v0.1.8)：✅**（域名+后缀通用过滤 → img标签提取 → scoped wrapper 统一扫描）
- **Phase 7.4 Edge CDP 兼容性修复 (v0.1.9-v0.1.12)：✅**（--new-window → taskkill三连 → auto_connect前杀进程，支持电脑小白用户）
- **Phase 7.5 Edge 导航与 JD 懒加载修复 (v0.1.13-v0.1.15)：✅**（移除 --new-window 冲突 + Windows 构建修复 + JD 详情容器强制展开）
- **Phase 8 抓取规则外置引擎 (v0.2.0)：✅**（解析逻辑从二进制剥离到可编辑规则包，平台改版免编译热更新；同步修复 JD 主图 + 天猫价格/规格/SKU）
- **Phase 8.1 双平台双架构构建矩阵 (v0.2.0)：✅**（macOS aarch64/x86_64 + Windows x64/ARM64，版本号统一对齐 0.2.0）
- **Phase 8.2 用户实测修复轮 (v0.2.1)：✅**（京东规则 v3/v4 两轮校准 + Windows 安装器修复：x64 只出 MSI / mainBinaryName=EGrab / NSIS perMachine）
- **CI/CD build+release 合并 (v0.1.5-v0.1.6)：✅**（解决 build/release 竞态，双平台自动发布）

---

## 任务状态

| # | 任务 | 负责人 | 状态 | 备注 |
|---|---|---|---|---|---|
| 1 | 项目骨架搭建（Tauri+Svelte） | backend+frontend | ✅ 已完成 | maintainer 已创建工程骨架 |
| 2 | CDP连接管理模块 | backend | ✅ 已完成 | src-tauri/src/cdp/ 已创建，cdp_connect/cdp_disconnect/cdp_status/cdp_list_tabs 已注册 |
| 3 | 淘宝商品页解析器 | backend | ✅ 已完成 | src-tauri/src/parser/taobao.rs 已创建，实现 PlatformParser trait |
| 4 | 京东商品页解析器 | backend | ✅ 已完成 | src-tauri/src/parser/jd.rs 已创建，实现 PlatformParser trait |
| 5 | 图片原图URL解析与批量下载 | backend | ✅ 已完成 | src-tauri/src/downloader/image.rs 568行完整实现（并发下载+重试+URL清洗） |
| 6 | 本地存档系统（SQLite+JSON+文件） | backend | ✅ 已完成 | src-tauri/src/storage/ 完整实现，含 7 个单元测试 |
| 7 | 基础UI界面 | frontend | ✅ 已完成 | 页面+组件+数据流全部完成并验证通过：UrlInput→onSubmit→Home→tasksStore.startScrape→currentTask→Progress；tsc 零错误 |
| 8 | 前后端IPC联调 | architect | ✅ 已完成 | 全部 11 个 IPC 命令已注册，83 个测试通过 |
| 9 | 双平台打包测试 | maintainer | ✅ 已完成 | macOS DMG 5.1MB 构建成功，reviewer 审计通过；Windows 构建修复：tauri.conf.json 已添加 icon.ico |
| 10 | 抓取引擎（scraper） | backend | ✅ 已完成 | src-tauri/src/scraper/engine.rs 实现完整抓取流程：CDP→Parser→Downloader→Storage |
| 11 | IPC 命令注册 | backend | ✅ 已完成 | start_scrape/cancel_scrape/get_task_history/get_task_detail/open_folder 全部已注册 |
| 12 | CI/CD build+release合并 | maintainer | ✅ 已完成 | build.yml 合并 release job，v0.1.5-v0.1.6 解决竞态同步 |
| 13 | JD解析器普适性增强（v0.1.5-v0.1.8） | planner | ✅ 已完成 | 路径白名单→域名+后缀→img标签提取→scoped wrapper 统一扫描 |
| 14 | Edge CDP兼容性修复（v0.1.9-v0.1.12） | planner | ✅ 已完成 | --new-window + taskkill三连 + auto_connect前杀进程 |
| 15 | Edge导航+JD懒加载修复（v0.1.13-v0.1.15） | planner | ✅ 已完成 | 移除 --new-window（与 new_page 冲突）+ wait_check_js 加 JD 选择器 + 详情容器强制展开 |
| 16 | 抓取规则外置引擎（v0.2.0） | planner | ✅ 已完成 | 新增 src-tauri/rules/ + parser/rules.rs；删除 jd.rs(727行)/taobao.rs(558行)；平台改版免编译 |
| 17 | JD/天猫解析故障修复（v0.2.0） | planner | ✅ 已完成 | JD 抗哈希子串选择器；天猫 price/specs/skus 取值路径修正（真实数据离线回归 8/8 通过） |
| 18 | 页面诊断快照能力（v0.2.0） | planner | ✅ 已完成 | dump_page_snapshot + raw.json counts 段 + 设置页三按钮 |
| 19 | 双平台双架构构建矩阵（v0.2.0） | planner | ✅ 已完成 | mac aarch64/x86_64 + win x64/ARM64；ARM64 标 continue-on-error 不阻断发布 |
| 20 | JD 规则 v3/v4 两轮校准（v0.2.1） | planner | ✅ 已完成 | v3 内容过滤(isJdProductImage/passSizeGate/分层 gallery/SKU 重写)；v4 基于真实 HTML 修 4 bug（imgzone 误杀/imagetools 拼写/scoped 过宽/播放图标）。离线回归 13/13 通过 |
| 21 | Windows 安装器修复（v0.2.1） | planner | ✅ 已完成 | x64 只出 MSI 消除双包冲突；mainBinaryName=EGrab；NSIS perMachine+installerIcon+中文；版本号对齐 0.2.1 |

---

## 阻塞项

| # | 阻塞项 | 严重度 | 影响范围 | 状态 |
|---|---|---|---|---|
| B-1 | `pre-mandate.md` "只运行一次"与变更传导协议死锁 | 致命 | 全局流程 | ✅ 已修复 — 人类授权后 pre 已修改 pre-mandate.md，明确"初始化运行一次 + 变更时经人类授权可重运行" |
| B-2 | `open_folder` IPC 命令无路径安全校验 | 致命 | 安全 | ✅ 已修复 — ipc-commands + storage-interface 已增加路径白名单约束 |
| B-3 | IPC 错误返回格式完全未定义 | 严重 | 前后端对接 | ✅ 已修复 — data-models.md 已增加 `IpcError`，ipc-commands.md 已补充错误语义 |
| B-4 | JSON 字段命名约定未指定 | 严重 | serde 配置 | ✅ 已修复 — data-models.md 已明确 snake_case + `#[serde(rename_all = "snake_case")]` |
| B-5 | `storage-interface.md` 类型引用缺失 | 严重 | TypeScript 编译 | ✅ 已修复 — 已补全 ProductData、TaskFilter、TaskSummary、TaskDetail import |
| B-6 | 去重检测无强制覆盖机制 | 严重 | 功能缺失 | ✅ 已修复 — `StartScrapeCommand.params` 已增加 `force?: boolean` |
| B-7 | `PageContext.raw_evaluate_result: unknown` | 严重 | backend 实现 | ✅ 已修复 — 已定义为 `JsonValue`，明确 Rust 对应 `serde_json::Value` |
| B-8 | 缺少 Downloader / CDP Manager / Scraper Engine 协议文档 | 严重 | 测试+实现基线缺失 | ✅ 已修复 — 新增 4 份协议：cdp-manager-interface.md、downloader-interface.md、scraper-engine-interface.md、config-interface.md |
| B-9 | ARCHITECTURE.md start_scrape 参数与 L4/L5 不一致 | 严重 | L2/L4 一致性 | ✅ 已修复 — 人类确认后 pre 已更新 ARCHITECTURE.md，start_scrape 签名与 L4/L5 完全一致 |
| B-10 | Windows 文件名安全规则未定义 | 严重 | 跨平台兼容 | ✅ 已修复 — pre 已在 storage-interface.md 和 downloader-interface.md 补强跨平台文件名安全规则 |
| **P0-1** | **Rust ErrorCode 序列化格式错误（snake_case vs SCREAMING_SNAKE_CASE）** | **致命** | **前后端联调** | **✅ 已修复** — architect 已修复并验证通过 |
| **P0-2** | **frontend opencode.json 权限过宽，可越权修改 src/protocols/** | **致命** | **L5 协议安全** | **✅ 已修复** — maintainer 已收窄权限并验证通过 |
| **P0-3** | **Tauri 工程骨架完全缺失（Cargo.toml/package.json/tauri.conf.json）** | **致命** | **项目编译** | **✅ 已修复** — maintainer 已创建完整工程骨架 |
| **P0-4** | **IpcError.code 类型为 String，未使用 ErrorCode 枚举** | **严重** | **类型安全** | **✅ 已修复** — architect 已修复并验证通过 |
| **P0-5** | **STATUS.md 存在过时引用（S-1、opencode.json 权限冲突）** | **严重** | **状态一致性** | **✅ 已修复** — planner 已更新 STATUS.md |
| **P0-6** | **tailwind.config.js 完全缺失，前端无法解析 Raycast UI 设计 Token** | **致命** | **前端构建** | **✅ 已修复** — planner 紧急创建，包含完整 Raycast 颜色/圆角/字体配置 |
| **P0-7** | **npx 命令缺少 --yes 参数导致交互提示阻塞 Tester 终端** | **严重** | **测试流程** | **✅ 已修复** — 已建立 npx --yes 铁律，所有后续测试命令必须加 --yes 跳过交互 |
| **P0-8** | **opencode.json 存在 3 处 deny 权限配置导致 Agent 阻塞** | **致命** | **全局调度** | **✅ 已修复** — 移除 frontend/qa/reviewer 的 deny 配置，改为 ask 通配符模式 |
| **P0-9** | **IpcError.code 协议文档类型双轨（ErrorCode vs ErrorCode\|string）** | **严重** | **L4/L5/L6 一致性** | **✅ 已修复** — pre 已将 L4/L5 中 `IpcError.code` 统一为 `ErrorCode` |
| **P0-10** | **opencode.json bash 白名单缺少 async_run.sh** | **严重** | **防阻塞机制** | **✅ 已修复** — 已增加 `./async_run.sh*` 和 `tail *` 到 bash 白名单 |

---

## 待修复项清单（延后到相关阶段处理）

> 以下问题已确认需要修复，但人类决策延后到相关开发阶段再处理。

| # | 问题 | 优先级 | 触发阶段 | 负责人 | 说明 |
|---|------|--------|----------|--------|------|
| D-1 | IpcResult<T> 与 Tauri Result<T, IpcError> 语义双轨 | P1 | Phase 5 IPC 联调 | pre/architect | 需明确 IPC 成功返回裸 T，服务层可包装为 IpcResult<T> |
| D-2 | open_folder 安全策略与 Tauri 2 capabilities 不匹配 | P1 | open_folder 实现前 | backend + maintainer | 需配置 opener 能力，reviewer 审计 |
| D-3 | force=true 重抓事务语义与 SQLite UNIQUE 约束 | P1 | storage 实现时 | backend + tester | 需验证事务测试 |
| D-4 | 淘宝/京东解析兜底链路 | P1 | ~~parser 开发前~~ | planner | ✅ **已由 v0.2.0 规则引擎（TD-009）解决** —— 多来源兜底逻辑下沉到规则脚本（ICE 路由遍历兜底 + DOM 兜底），Spike 2 作废 |
| D-5 | STATUS.md / TECH_BOARD "done" 表述过早 | P1 | Phase 5 状态更新 | planner + architect | CDP 模块状态需修正为"基础实现完成/真实联调待验证" |
| D-6 | tailwind.config.js 缺少 ss03 字体特性 | P2 | UI 开发阶段 | frontend/maintainer | 需补齐 Raycast 设计规范 |
| D-7 | tauri.conf.json CSP 与 bundle targets | P2 | 发布前 | maintainer | 需配置 CSP 和明确 bundle targets |
| D-8 | src/protocols/README.md 文件清单过时 | P2 | 文档维护 | architect | 需更新协议文件清单 |
| D-9 | AGENTS.md §5 未要求 reviewer 读取 src/protocols/ | P2 | 文档维护 | pre | 需补充 reviewer 读取要求 |

---

## 当前进行中事项（v0.2.1 发布后，更新于 2026-09-01）

| # | 事项 | 优先级 | 状态 | 说明 |
|---|------|--------|------|------|
| C-1 | v0.2.0 CI 首次编译验证 | P0 | ✅ 已完成 | CI 全绿，5 个产物全部构建成功（含实验性 Windows ARM64），Rust 盲写 1053 行一次编译通过，体积全部 <15MB |
| C-2 | JD 主图/价格选择器校准 | P0 | ✅ 已完成 | 经 v3（盲写过滤）+ v4（真实 HTML 校准）两轮修复，离线回归 13/13 通过；待用户真机最终确认 counts 段 |
| C-3 | Windows ARM64 构建可行性 | P2 | ✅ 已完成 | 交叉编译 + NSIS 打包成功。附带发现：用户曾把 ARM64 包装到 x64 机器上导致"空白图标+打不开"，属架构选错非杀软问题 |
| C-4 | 规则包版本升级覆盖用户改动 | P2 | 📋 已文档化 | 内嵌 version 高于磁盘时会备份 `*.bak` 后覆盖。用户如需固定自改规则，须将 `rules.json` 的 `version` 改为大数（如 9999）。已写入 `rules/README.md` |
| C-5 | 未提交的单 Agent 模式文档迁移 | P2 | 📋 待用户决策 | 工作区存在 AGENTS.md/opencode.json/contract-*.md/src\/protocols/README.md 的模式迁移改动 + 269 个已删除日志证据文件 + AGENTS.md.multi/.single/.bak*、docs/PLANNER_HANDBOOK.md 等未跟踪文件，尚未提交。**注意：v0.2.1 中 STATUS/TECH_BOARD/HISTORY 的文档更新已单独提交，不受此项阻塞** |
| C-6 | v0.2.1 CI 构建 + 用户真机验证 | P0 | ⏳ 等待 CI/用户 | tag v0.2.1 已推送触发 CI；待用户：①装 v0.2.1 验证快捷方式/图标正常 ②抓京东商品看 raw.json counts 段（预期 gallery=5、detail_images=12、price=87 量级）③卸载 %LOCALAPPDATA%\EGrab 的错架构残留 |

---

## 全局一致性双检结论

**总体评级：有条件通过**

双检报告汇总：
- **pre 自检**：不通过（已修复 4 个 contract + 补齐 4 个 L5 协议 + 修复 storage.ts；仍有 opencode.json 冲突、L2 参数冲突、Rust 类型问题超出权限）
- **reviewer 独立审计**：有条件通过（发现 opencode.json 权限冲突、tests/ 目录缺失、ConnectionInfo 类型不一致）

**关键发现**：
1. 命名一致性（ProductData 九字段、IPC 命令、事件名）全链路零错漏 ✓
2. 指挥链一致性已修复（contract 工作流与 AGENTS.md 对齐）✓
3. L5 缺失的 4 份 TypeScript 协议已补齐 ✓
4. reviewer 定位已从"代码审计"扩展为"一致性审计"✓
5. **仍阻塞**：opencode.json 6 处权限冲突、ARCHITECTURE.md start_scrape 参数与 L4/L5 不一致

**建议下一步**：
1. 人类授权修复 opencode.json 6 处权限冲突
2. 人类确认 ARCHITECTURE.md `start_scrape` 是否正式加入 `force?` 参数
3. architect 后续修复 Rust 类型严格性问题
4. 修复后进入 Phase 5（前后端开发）

---

## 全局一致性审核结论（第二轮 - 2026-05-09）

**总体评级：有条件通过**

**连通性测试结果**：11/11 agent 全部连通成功 ✅

**各agent审核结论汇总**：

| Agent | 模型 | 结论 | 评分 | 关键发现 |
|-------|------|------|------|----------|
| architect | zhipuai/glm-5.1 | 有条件通过 | 7.5/10 | ErrorCode serde 序列化格式错误（致命） |
| reviewer | minimax-cn/MiniMax-M2.7 | 有条件通过 | 7.5/10 | reviewer edit 权限配置问题 |
| qa | openrouter/anthropic/claude-opus-4.7 | 可行（需调整） | 8.2/10 | chromiumoxide+Tauri runtime 整合陷阱、15MB 体积、淘宝解析路径 |
| frontend | zhipuai/glm-5v-turbo | 可行 | 8/10 | README 文件清单过时、ScrapeErrorPayload.error 为 string |
| backend | deepseek/deepseek-v4-pro | 需调整 | 3.5/10 | Tauri 工程骨架完全缺失（致命） |
| tester | moonshotai/kimi-k2.6 | 可行（需调整） | 6/10 | 测试基础设施缺失、CDP mock 方案空白 |
| maintainer | stepfun/step-3.5-flash | 需调整 | 3/10 | Cargo.toml/package.json/tauri.conf.json 完全缺失 |
| history | deepseek/deepseek-v4-flash | 需补充 | 8/10 | HISTORY.md 日期排序异常、STATUS.md 过时引用 |
| fallback | alibaba/qwen3.6-max-preview | 有风险 | 7/10 | ErrorCode 序列化与协议完全不一致（致命） |

**Pre 自审结论**：有条件通过，L3/L4 一致性评分 7.2/10

**P0 阻塞项（必须修复后方可进入 Phase 5）**：
1. Rust ErrorCode 序列化格式错误（snake_case vs SCREAMING_SNAKE_CASE）
2. frontend opencode.json 权限过宽，可越权修改 src/protocols/
3. Tauri 工程骨架完全缺失（Cargo.toml/package.json/tauri.conf.json）
4. IpcError.code 类型为 String，未使用 ErrorCode 枚举

**建议下一步**：
1. architect 修复 Rust L5 错误模型（ErrorCode/IpcError/DuplicateTaskConflict）
2. planner/maintainer 修复 opencode frontend 权限
3. maintainer 建立最小工程骨架
4. 修复完成后进入 Phase 5（前后端开发）

---

## kimi-k2.6 成功态不收敛问题分析

### 问题现象
kimi-k2.6（tester）在执行异步任务时，反复出现"成功态不收敛"问题：
1. 启动 cargo test 和 tsc 两个异步任务
2. tsc 立即完成（STATE=FINISHED, EXIT_CODE=0）
3. cargo test 还在运行（STATE=RUNNING）
4. kimi-k2.6 检查状态文件 3 次（每次都是 RUNNING），然后卡住
5. 既没有检查日志确认编译状态，也没有汇报部分结果（tsc 已通过）

### 根本原因分析
1. **模型能力限制**：kimi-k2.6 可能不擅长处理复杂的异步状态机和条件分支
2. **提示词过于复杂**：contract-tester.md 有 244 行，包含大量规则，kimi-k2.6 可能无法全部理解
3. **temperature 过高**：temperature=1 可能导致输出随机性增加，不利于遵循严格的规则
4. **异步任务结果判定逻辑复杂**：需要结合 .status 文件、日志、进程状态综合判断

### 违反的规则
1. **不可判定状态升级铁律**：检查了 3 次状态文件（超过了 2 次限制），仍不能确认结果，应该立即升级
2. **强制收敛状态机**：看到 STATE=FINISHED（tsc）后，应该立即进入状态 C（成功收敛），输出汇报
3. **部分完成概念**：tsc 已经成功，这是部分结果，应该立即汇报

### 解决方案（优先级排序）
1. **降低 temperature**：将 tester 的 temperature 从 1 降低到 0.3（❌ 无效 — kimi-k2.6 不支持 temperature 自定义）
2. **简化提示词**：为 kimi-k2.6 创建更简洁的提示词，专注于核心规则（延后）
3. **增加明确的指令**：在任务提示中明确说明"如果 cargo test 还在运行，立即汇报部分结果"（延后）
4. **更换模型**：✅ 已实施 — kimi-k2.6 移至 fallback（temperature=1），tester 改用 step-3.5-flash，maintainer 改用 qwen3.6-plus

### 建议下一步
1. 立即将 tester 的 temperature 从 1 降低到 0.3（已完成）
2. 为 kimi-k2.6 创建简化版的提示词（重点强调异步任务处理）
3. 在下次调度 tester 时，在任务提示中明确说明"部分完成"的处理方式
4. 如果问题仍然存在，考虑更换 tester 模型

---

## 决策记录

| 日期 | 决策 | 决策者 |
|------|------|--------|
| 2026-05-05 | 项目启动，确认技术栈为 Tauri+Svelte+Rust | 人类 |
| 2026-05-05 | MVP范围确认为淘宝+京东双平台基础抓取 | 人类 |
| 2026-05-08 | pre agent 完成制宪（10 contract + 5 protocol） | pre |
| 2026-05-08 | 五方审计完成（qa/fallback/architect/tester/reviewer） | planner |
| 2026-05-08 | 审计结论：有条件通过，需修复 2 致命 + 6 严重缺陷后方可进入开发 | planner |
| 2026-05-08 | pre agent 完成自检修复：修复 8/9 项阻塞（B-2~B-8），新增 4 份协议文档；B-1 为 L0 权限外问题，给出修复建议待人类确认 | pre |
| 2026-05-09 | 人类授权 maintainer 放权：pre 同步更新 AGENTS.md + contract-maintainer.md + opencode.json | 人类 + pre |
| 2026-05-09 | 人类授权启动 Phase 4：architect 生成 src/protocols/（6 TypeScript 文件）和 src-tauri/src/models/（5 Rust 文件） | 人类 + architect |
| 2026-05-09 | 人类授权修改工作流：pre 更新全部 10 个 contract，新增开发-测试-审计闭环、运维接棒机制、卡点兜底、qa 随时调用权限 | 人类 + pre |
| 2026-05-09 | 全局一致性双检完成：pre 自检 + reviewer 独立审计。发现 opencode.json 6 处权限冲突、L5 缺失 4 份协议（已补齐）、storage.ts 与 L4 不一致（已修复） | planner + pre + reviewer |
| 2026-05-09 | 人类确认四项决策：start_scrape 加入 force?、reviewer 双入口（方案A）、全部 contract 新增上下文窗口自知力、pre-mandate 已修改无需重跑 | 人类 |
| 2026-05-09 | 全局一致性调整落地：AGENTS.md 新增 reviewer 双入口、ARCHITECTURE.md start_scrape 加入 force?、opencode.json 修复 6 处权限冲突 + 更新 description、全部 10 个 contract 新增上下文窗口章节 | pre + planner |
| 2026-05-09 | 模型互换：planner 从 kimi-k2.6(262K) 换为 mimo-v2.5-pro(1M)，tester 反向互换；同步更新 opencode.json + 对应 contract 上下文窗口章节 | 人类 + pre |
| 2026-05-09 | 模型更换后全局审核完成：连通性测试 10/11 agent 成功（tester 响应为空）；reviewer/architect/qa/pre 四方审核；pre 已修复协议层问题（README 状态表、contract 编号、Windows 文件名规则、模型 ID 一致性） | planner + reviewer + architect + qa + pre |
| 2026-05-09 | 人类确认 ARCHITECTURE.md 加入 force? 参数，pre 已修复 B-9；architect 已补全 TECH_BOARD.md 模块表 + Spike 验证规划 | 人类 + pre + architect |
| 2026-05-09 | **模型更换后全局审核（第二轮）完成**：11/11 agent 连通性测试全部成功；9 agent 一致性审核；pre 自审。发现 4 项 P0 阻塞（ErrorCode 序列化、frontend 权限、工程骨架、IpcError 类型） | planner + pre + 9 agent |
| 2026-05-09 | **P0 阻塞修复完成**：architect 修复 ErrorCode 序列化和 IpcError 类型；maintainer 修复 opencode.json 权限和创建工程骨架；reviewer 验证通过 | architect + maintainer + reviewer |
| 2026-05-09 | **Phase 5 启动**：所有 P0 阻塞已清零，正式进入前后端开发阶段 | planner |
| 2026-05-09 | **UI 设计规范确立**：采用 Raycast 风格，纯暗黑模式、无阴影、Hairline 边框、ss03 字体特性、主按钮纯白。已保存 DESIGN.md | 人类 + planner |
| 2026-05-09 | **防阻塞铁律确立**：重型依赖下载必须使用国内镜像源（npm: npmmirror, cargo: rsproxy）；长耗时命令必须后台执行 | 人类 + planner |
| 2026-05-09 | **DESIGN.md 加载策略调整**：从全局 instructions 移除，改为仅 UI/前端相关角色（frontend/architect）按需读取原文。核心要素已写入 AGENTS.md、contract-frontend.md、contract-architect.md | pre + planner |
| 2026-05-09 | **History 归档工作流最终确立**：经历 3 轮迭代（浓缩摘要→仅问题原文→完整问题+回答原文），AGENTS.md §3.5 新增 History 归档铁律，contract-planner.md §8.4 和 contract-history.md §2 同步更新 | pre + planner |
| 2026-05-09 | **紧急修复 P0-6**：tailwind.config.js 完全缺失导致前端构建失败。planner 紧急创建，包含完整 Raycast 颜色/圆角/字体配置 | planner |
| 2026-05-09 | **紧急修复 P0-7**：npx 命令缺少 --yes 参数导致 Tester 终端交互阻塞。确立 npx --yes 铁律 + 后台执行铁律 | planner + 人类 |
| 2026-05-09 | **紧急修复 P0-8**：opencode.json 存在 3 处 deny 权限配置（frontend/qa/reviewer），导致 Agent 调度阻塞。全部改为 ask 通配符模式 | planner |
| 2026-05-10 | **全局一致性审查（Phase 4.9）**：pre 审查全部 10 份 contract + opencode.json + AGENTS.md，修复：① npx 白名单更新为 --yes 形式；② AGENTS.md §4 增加权限说明；③ 10 份 contract 同步防阻塞铁律 | pre + planner |
| 2026-05-10 | **紧急系统级干预**：Frontend/Backend 再次发生长耗时命令阻塞。planner 创建 async_run.sh 物理级防阻塞脚本，升级防阻塞铁律为"绝对禁止直接执行"，要求所有长耗时命令必须通过 async_run.sh 执行并进行日志抽查 | planner + 人类 |
| 2026-05-10 | **全局规范持久化（重启准备）**：pre 将三大铁律（防阻塞与异步铁律、包管理铁律、分工机制铁律）永久写入 AGENTS.md，确保重启后上下文不丢失 | pre + planner |
| 2026-05-10 | **opencode.json 参数配置分析**：经搜索 opencode 源码和 schema，确认 `reasoningEffort` 参数仅对 OpenAI 和 Local provider 的模型有效。当前项目使用的 provider（openrouter/moonshotai/zhipuai/deepseek/xiaomi/minimax-cn/stepfun/alibaba）不支持推理强度配置 | planner |
| 2026-05-10 | **三大铁律同步到所有角色 contract**：pre 已将防阻塞与异步铁律、包管理铁律、分工机制铁律写入全部 10 个 contract 文件和 pre-mandate.md，确保每个角色启动时都能加载到关键规范 | pre |
| 2026-05-10 | **各角色铁律针对性优化**：pre 根据角色特点优化了三大铁律内容。planner/architect/history/qa 简化为调度层原则；frontend 重点强调 npm/npx；backend 重点强调 cargo；tester/maintainer 完整保留；reviewer 简化为审计检查项；fallback 完整保留并强化紧急约束 | pre |
| 2026-05-10 | **全局审计完成**：planner/reviewer/qa/pre 四方审计，发现 IpcError.code 类型双轨（P0）、opencode.json 缺少 async_run.sh 白名单（P0）等问题。已修复 P0 项，P1/P2 延后到相关阶段处理 | planner + reviewer + qa + pre |
| 2026-05-10 | **P0 修复完成**：① IpcError.code 协议文档已从 `ErrorCode | string` 改为 `ErrorCode`；② opencode.json bash 白名单已增加 `./async_run.sh*` 和 `tail *` | pre + planner |
| 2026-05-10 | **opencode.json 子 agent 权限同步**：所有 10 个子 agent（pre/planner/architect/frontend/backend/tester/qa/history/reviewer/maintainer）的 permission.bash 已增加 `./async_run.sh*` 和 `tail *` 的 allow 权限 | planner |
| 2026-05-10 | **Phase 5 后端核心模块全部完成**：architect 完成模块接入（storage/parser/downloader/scraper 接入 lib.rs）+ 缺失模块开发（taobao.rs/jd.rs/scraper engine/scrape_commands/task_commands）。cargo check 零错误，cargo test 83/83 全部通过，11 个 IPC 命令全部注册 | architect |
| 2026-05-10 | **紧急系统级干预 — 异步结果不可判定型逻辑阻塞**：Frontend 执行 `npx --yes tsc --noEmit` 后因 tsc.log 为空（成功时无输出是正常行为）进入逻辑自旋。根因是旧版 async_run.sh 缺少 exit code / status / pid 可观测性 | 人类 + planner |
| 2026-05-10 | **async_run.sh v2 升级完成**：新增 `.status` 文件（STATE/EXIT_CODE/FINISHED_AT）和 `.pid` 文件，实现异步任务状态可观测。planner 已验证空输出命令场景下 status 文件正确写入 | planner |
| 2026-05-10 | **不可判定状态升级铁律持久化**：pre 将异步任务结果判定铁律、不可判定状态升级铁律、上级/QA 求助机制写入 AGENTS.md §8.1.1/§8.1.2 和全部角色 contract | pre + planner |
| 2026-05-10 | **通用智能防阻塞意识持久化**：pre 将通用智能防阻塞意识铁律、智能防阻塞判定流程、BLOCKED_REPORT 标准阻塞报告协议写入 AGENTS.md 和全部 10 个角色 contract，让所有 Agent 具备自发的、普适性的防阻塞能力 | pre + planner |
| 2026-05-10 | **Phase 5 前端数据流验证完成**：重启后 planner 验证前端数据绑定已正确实现——UrlInput→onSubmit→Home→tasksStore.startScrape→currentTask→Progress 全链路正确；tsc --noEmit 零错误（async_run.sh v2 确认 STATE=FINISHED, EXIT_CODE=0）。此前 Frontend 自旋阻塞的根因是旧版 async_run.sh 缺乏可观测性，代码本身已正确 | planner |
| 2026-05-10 | **Phase 5 收尾 — architect 技术评估**：architect 验证前端数据流完整性（页面→stores→services→IPC 全链路无越层调用），更新 TECH_BOARD.md（P5-3/P5-8 标记完成，接口联调状态全部更新），TypeScript 最终验证零错误。结论：Phase 5 可以关闭 | architect |
| 2026-05-10 | **Phase 5 收尾 — reviewer 一致性审计**：reviewer 审计命名一致性（L1/L2/L4/L5/L6 全链路一致）、接口一致性（TypeScript/Rust 类型与协议对齐）、数据流一致性（组件→stores→services 分层清晰）、权限一致性（opencode.json 符合 AGENTS.md §4）。结论：Phase 5 一致性审计通过，可以正式关闭 | reviewer |
| 2026-05-10 | **Phase 6 批次 1 完成 — 模型序列化测试**：architect 调度 tester 执行 P6-1，新增 37 个测试（models_serde_test 19 个 + cdp_state_test 12 个 + commands_test 6 个），总测试数 43/43 通过，EXIT_CODE=0。覆盖 ProductData/ConnectionState/AppConfig/TaskFilter/TaskStatus/ScrapeStep 等核心模型的 serde round-trip | architect + tester |
| 2026-05-10 | **Tester（kimi-k2.6）成功态不收敛问题（第二次）**：tester 已确认测试通过但未汇报，继续执行 rm 清理命令后卡住。planner 强制接管，由 architect 直接基于已有结果汇报。kimi-k2.6 模型已两次出现此问题，建议后续考虑模型更换或增加强制收敛提示词 | planner |
| 2026-05-10 | **终态收敛与证据保全铁律持久化**：planner 调度 pre 将四大铁律写入 AGENTS.md（§8.4-8.7）和全部 10 份角色 contract：① 终态收敛铁律（明确终态后禁止继续执行无关命令）② 证据保全铁律（禁止在上级验收前删除 log/status/pid）③ 报告优先铁律（产生汇报意图后必须立即输出报告）④ Tester/QA 标准报告格式。通信测试验证 architect 和 tester 链路均正常 | pre + planner |
| 2026-05-10 | **Phase 6 批次 2/3 完成 — 存储引擎集成 + 抓取引擎 E2E**：architect 调度 tester 并行执行 P6-2（storage_integration_test.rs 6 个测试）和 P6-3（scraper_test.rs 5 个测试）。全量测试 131/131 通过，EXIT_CODE=0。Tester（kimi-k2.6）第三次出现成功态不收敛，planner 强制接管确认结果 | architect + tester |
| 2026-05-10 | **Phase 6 正式关闭**：P6-1 模型序列化（43/43）+ P6-2 存储引擎集成（6/6）+ P6-3 抓取引擎 E2E（5/5）全部通过，全量 131 tests passed。Phase 6 测试联调 100% 完成 | planner |
| 2026-05-10 | **Tester 强制收敛状态机持久化**：针对 kimi-k2.6 成功态不收敛问题（3 次），planner 调度 pre 在 contract-tester.md 新增"强制收敛状态机"章节：状态 A（执行中）→ 状态 B（检查结果）→ 状态 C（成功收敛，强制输出汇报）/ 状态 D（失败收敛）。核心机制：看到 STATE=FINISHED 后唯一允许动作是输出 TEST_REPORT，禁止执行任何其他命令 | pre + planner |
| 2026-05-10 | **kimi-k2.6 第四次成功态不收敛**：architect 调度 tester 验证 Phase 6 全量测试，kimi-k2.6 启动 cargo test 和 tsc 后，tsc 立即完成（EXIT_CODE=0），cargo test 还在运行（STATE=RUNNING）。kimi-k2.6 检查状态文件 3 次（每次都是 RUNNING），然后卡住，既没有检查日志确认编译状态，也没有汇报部分结果（tsc 已通过）。违反了"不可判定状态升级铁律"（最多检查 2 次）和"强制收敛状态机"（看到 FINISHED 必须汇报）。planner 强制接管，确认 cargo test 和 tsc 均已通过（36/36 测试，EXIT_CODE=0） | planner |
| 2026-05-10 | **kimi-k2.6 问题分析与解决方案**：分析根本原因（模型能力限制、提示词复杂、temperature 过高、异步逻辑复杂），制定四个解决方案：① 降低 temperature（已实施，从 1 降到 0.3）；② 简化提示词；③ 增加明确指令；④ 考虑更换模型。更新 STATUS.md 记录分析结果 | planner |
| 2026-05-10 | **Phase 5+6 全面检查验证完成**：architect 完成全面检查测试与验证：代码质量良好、131 测试全通过、一致性审计通过。Phase 7 可启动 | architect + planner |
| 2026-05-10 | **三角色模型变更**：人类确认 kimi-k2.6 因成功态不收敛（4次）且不支持 temperature 自定义，从 tester 移至 fallback（temperature=1）；tester 改用 stepfun/step-3.5-flash（轻量快速，适合执行型任务）；maintainer 改用 alibaba/qwen3.6-plus（1M 上下文，适合运维复杂配置）。opencode.json + 3 份 contract 已同步更新 | 人类 + planner + pre |
| 2026-05-10 | **Phase 7 打包交付完成**：maintainer 完成 macOS 生产构建（DMG 5.1MB，远低于 15MB 限制），修复 2 项配置问题（identifier 改为 com.egrab.desktop 避免 .app 冲突，移除损坏的 icns/ico 图标文件）。reviewer 运维/配置审计全部通过（配置变更合理、依赖一致、安全配置无问题）。MVP-1 全部 7 个 Phase 完成 | maintainer + reviewer + planner |
| 2026-05-11 | **京东解析器修复**：用户报告 detail_images 和 specs 为空。backend 修复：① 详情图片改用 `getComputedStyle()` 提取 CSS background-image（原代码只检查内联 style，但京东样式在 `<style>` 标签中通过 class 定义）；② avif→jpg 格式转换重构（先转格式再移除尺寸前缀）；③ 扩展查询范围到 `#detail-main` 和 `#detail-top`。139/139 测试通过 | backend + planner |
| 2026-05-13 | **Windows CI 构建修复**：tauri.conf.json 中 bundle.icon 缺少 .ico 格式图标，导致 Windows MSI 打包失败（`Couldn't find a .ico icon`）。maintainer 在 icon 数组中添加 `"icons/icon.ico"`。文件已存在于 `src-tauri/icons/icon.ico`，仅需配置引用 | maintainer + planner |
| 2026-05-13 | **CI 自动 Release**：maintainer 在 build.yml 新增 release job，推送 v* 标签时自动创建 GitHub Release 并附加 macOS DMG + Windows MSI。已推送 v0.1.0 标签触发首次 Release | maintainer + planner |
| 2026-05-13 | **懒加载滚动修复（detail 目录为空）**：根因是 chromiumoxide `page.evaluate()` 默认 `awaitPromise: false`，导致 JS `new Promise(...)` 的 setInterval/scrollTo 未执行完就被丢弃。改为 Rust 侧 `tokio::time::sleep` 循环 + 同步 JS。修复影响全平台 | backend + planner |
| 2026-05-18 | **JD 解析器三轮普适性增强（v0.1.5→v0.1.8）**：① 路径白名单→域名+后缀通用过滤；② 新增 `<img src>` 标签提取（书籍/茶叶/iPhone 等不用 CSS 背景图的商品）；③ 从硬编码 ID 列表改为 `_scoped_1nhp8_1` 根容器统一扫描。覆盖京东全部详情图形式 | planner |
| 2026-05-18 | **CI build+release 合并（v0.1.5→v0.1.6）**：解决旧方案中 build.yml 和 release.yml 竞态导致的 Release 无附件问题。合并后 build 完成才触发 release，v0.1.6 起 5 个 Assets 全部同步 | planner |
| 2026-06-11 | **Edge CDP 兼容性五轮迭代（v0.1.9→v0.1.12）**：① --new-window 强制弹窗；② taskkill 三连杀（每次间隔 2s 对抗 Startup Boost）；③ 回退 --disable-background-mode / --disable-features（部分 Edge 版本会阻止窗口打开）；④ 最终方案：auto_connect 启动前先 kill_browser_process，确保 Edge 后台进程不会阻止 CDP 端口绑定 | planner |
| 2026-06-11 | **360 杀毒软件误报**：用户反馈 EGrab 被 360 标记为"银狐木马"。分析为未签名 exe + CDP 浏览器连接 + 网络下载三个行为特征触发的误报。长期方案：购买 Windows Authenticode 代码签名证书 | planner |
| 2026-06-11 | **GitHub 仓库可见性**：用户决定隐藏仓库。确认免费私有仓库可用，因 gh CLI 需要 OAuth 认证且本地无 token，建议用户通过网页端操作（Settings → Danger Zone → Change visibility） | planner |
| 2026-06-18 | **Edge 导航修复（v0.1.13）**：`--new-window` 与 CDP `new_page()` 冲突导致 Edge 导航失败，移除该 flag；`wait_check_js` 补充 JD 专属选择器（`#detail-main`/`#detail-top`/`_scoped_1nhp8_1`/`.sku-title-name`），避免 Edge 干等 Taobao-only 元素超时；杀进程后加 1s 冷却 | planner |
| 2026-06-18 | **Windows 构建修复（v0.1.14）**：cdp 模块缺失 `cmd` 声明导致 Windows 编译失败，补回 | planner |
| 2026-06-18 | **JD 详情容器强制展开（v0.1.15）**：JD 详情容器用 固定 height + overflow:hidden + transform:scale，导致 `window.scrollTo` 触达不到图片元素。滚动前注入 JS 解除高度/溢出限制并将 img 设为 `loading=eager` | planner |
| 2026-08-29 | **故障定位（基于用户提供的真实 raw.json）**：① JD `pageConfig.product` 已被平台清空，DOM 兜底的 `._gallery_116km_1` 是构建哈希类名，发版即失效 → GALLERY_EMPTY；② 天猫数据源仍在但取值路径全部错位——价格实际在 `skuCore.sku2info[skuId].price.priceMoney`（分）、规格在 `plusViewVO.industryParamVO.basicParamList`、SKU 在 `skuBase.props`+`skuCore.sku2info`。结论：把易变解析规则编译进二进制是架构错误 | planner |
| 2026-08-29 | **抓取规则外置引擎落地（v0.2.0，TD-009）**：新增 `src-tauri/rules/`（rules.json + 各平台 extract/expand JS），随二进制 `include_str!` 内嵌并于首次启动释放到 `<app_data>/com.egrab.app/rules/`。磁盘优先、解析失败自动回退内嵌、每次抓取重新读盘、内嵌版本更高时备份 `*.bak` 后升级、脚本文件名做目录穿越校验。新增 `parser/rules.rs` 通用规则驱动解析器，删除 `parser/jd.rs`(727行) 与 `parser/taobao.rs`(558行)，Rust 侧不再含任何平台专属逻辑 | planner |
| 2026-08-29 | **解析故障修复（v0.2.0）**：天猫 price/specs/skus 三处取值路径修正，用用户提供的真实页面数据构造 fixture 做离线回归，8/8 断言通过（price 168.06~198、specs 7 条、sku price=198 stock=200）；JD 全面改用 `[class*="gallery"]`/`[class*="carousel"]`/`[class*="scoped"]` 子串选择器抵抗构建哈希变化 | planner |
| 2026-08-29 | **诊断闭环建立（v0.2.0）**：新增 `dump_page_snapshot`（导出完整 DOM + 12 个候选全局变量 + 图片/id/class 清单）、`get_rules_info`、`reload_rules`、`open_rules_folder` 四个 IPC；`raw.json` 新增 `counts` 段；新增 `PRICE_MISSING`/`SPECS_EMPTY`/`EXTRACT_JS_RETURNED_NULL` 降级告警码；设置页新增三个运维按钮 | planner |
| 2026-08-29 | **双平台双架构构建矩阵（v0.2.0，TD-010）**：macOS aarch64+x86_64（DMG，改 matrix 并行 + 文件名加架构后缀）、Windows x86_64（MSI+NSIS）、Windows aarch64（NSIS，WiX 在 ARM64 不稳定故只出 NSIS）。ARM64 job 标 `continue-on-error`，release 条件改为 `always() && needs.build-macos.result=='success'`，实验性架构失败不阻断其余产物发布。新增 cargo 缓存。版本号统一对齐 0.2.0（此前 4 个文件一直停留 0.1.0，导致所有 tag 产物文件名都叫 EGrab_0.1.0） | planner |
| 2026-08-29 | **本地无 Rust 工具链下的验证策略**：确认本机 `cargo`/`rustc` 均不存在，遵循 AGENTS.md §8.8「本地不做编译，走 GitHub CI」。采用三级离线验证替代：① Node 校验 rules.json 合法性 + 4 个 JS 文件语法；② 真实数据 fixture 跑 extract 脚本做回归断言；③ `npx tsc --noEmit` 零错误。Rust 首次编译由 CI 承担 | planner |
| 2026-09-01 | **v0.2.0 用户实测 + JD 二次失灵定位**：用户在 Windows 真机测试。天猫完全恢复（gallery 5/detail 11/skus 1/specs 36/price 192.06~198）。JD item 1444522840 仍故障：gallery 命中 35 张含大量图标、cover 取到图标、detail 36 张混入推荐位、skus 为 0。结论：v0.2.0 选择器"抗哈希但过宽"，需内容过滤而非继续收窄 | planner |
| 2026-09-01 | **京东规则 v3（commit 13fc506）**：加内容过滤层——isJdProductImage（jfs/ 路径 + 噪声路径黑名单）、passSizeGate（<150px 拒收但放行 sNxN_jfs 缩略图）、inExcludedZone（recommend/comment/shop/banner 等排除区）、gallery 5 组分层第一组命中即停、封面独立取大图、SKU 重写覆盖新旧规格结构、详情 wrapper 排除区过滤。自诊断字段 gallerySample/coverUrl/rejectedSample 等。实测 item 10026681425538：gallery 2（仍混入 1 张视频播放图标）、detail 11（混入 2 张小图标）、price 87 正确、sku 1 | planner |
| 2026-09-01 | **用户提供真实京东 HTML → 规则 v4（commit a12f306）**：基于完整 DOM 定位 4 个真 bug——① imgzone 被误列噪声路径（详情长图恰恰全在 imgzone/jfs/，v3 会全滤掉）② imgtools 拼写错误（实际 imagetools，导致播放图标漏网）③ [class*=scoped] 详情容器过宽（13 个匹配混入缩略图/图标）④ gallery 需排除 img.thumbnails-play-icon。修复后按真实 DOM 还原 stub 做离线回归 13/13 全过（gallery 5 去重无图标/封面 s1440x1440/详情 12 张全 imgzone/价格 87/SKU 带图）。规则包版本 3→4 | planner |
| 2026-09-01 | **Windows 快捷方式失灵排查**：用户报告快捷方式无图标且打不开、目标指向 %LOCALAPPDATA%\EGrab\egrab.exe。排查结论：① v0.2.0 首次同时产出 MSI+NSIS 两个 x64 包，装到不同目录互相冲突，卸载其一留下死快捷方式 ② 用户另把 ARM64 包装到 x64 机器（文件存在但无法执行，空白图标+双击无反应），属架构选错而非杀软 ③ %LOCALAPPDATA% 未签名 exe 是杀软重点拦截对象。修复（v0.2.1，commit b6b9ef0）：x64 只出 MSI、mainBinaryName=EGrab、NSIS perMachine+installerIcon+简体中文、版本号对齐 0.2.1 | planner |
| 2026-09-01 | **复盘结论（京东为何"之前能抓后来失灵"）**：京东 6 月→9 月一次前端改版动了两个层面——数据层（pageConfig.product 被清空）+ 表现层（CSS-Module 哈希类名变化），v0.2.0 前的解析器恰好同时依赖这两者，所以主图/价格/店铺一起丢；同期用稳定 ID（#detail-main）的详情图和规格存活，反向验证根因。架构教训（TD-009）：真正的故障不是某个选择器失效，而是"修选择器的成本太高"（改 Rust→CI 编译→重发安装包→全员重装），v0.2.0 规则外置后修复成本降为改一个文本文件 | planner |

---

## Agent 防阻塞规则状态

> 本章节记录当前已持久化的防阻塞规则体系。

| 规则 | 状态 | 写入位置 |
|------|------|---------|
| 通用智能防阻塞意识铁律 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| 智能防阻塞判定流程（常识→有限检查→升级） | ✅ 已持久化 | AGENTS.md + 全部 contract |
| BLOCKED_REPORT 标准阻塞报告协议 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| 不可判定状态最多检查 2 次规则 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| 上级/QA 求助机制 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| 异步任务必须结合 status/pid/log 判断 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| async_run.sh v2（.status + .pid 文件） | ✅ 已升级 | async_run.sh |
| `npx --yes tsc --noEmit` 成功时可能空输出 | ✅ 已强调 | AGENTS.md |
| npm 镜像源铁律 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| cargo 镜像源铁律 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| npx --yes 铁律 | ✅ 已持久化 | AGENTS.md + 全部 contract |
| 终态收敛铁律 | ✅ 已持久化 | AGENTS.md §8.4 + 全部 contract |
| 证据保全铁律 | ✅ 已持久化 | AGENTS.md §8.5 + 全部 contract |
| 报告优先铁律 | ✅ 已持久化 | AGENTS.md §8.6 + 全部 contract |
| Tester/QA 标准报告格式 | ✅ 已持久化 | AGENTS.md §8.7 + tester/qa contract |

**当前目标**：防止所有 Agent 在日志为空、命令无输出、状态不明时自旋卡死；防止成功态不收敛、证据销毁、汇报延迟

---

## 全局防阻塞铁律（不可违反）

> 本章节定义了防止 Agent 调度链卡死的强制规范，所有 Agent 必须严格遵守。

### 1. 依赖下载必须使用国内镜像源

**npm 相关**：
```bash
# 方式一：命令行指定（推荐）
npm install --registry=https://registry.npmmirror.com

# 方式二：配置 .npmrc
npm config set registry https://registry.npmmirror.com
```

**cargo 相关**：
```toml
# ~/.cargo/config.toml
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
```

### 2. 长耗时命令必须后台执行（物理级防阻塞铁律）

**【绝对铁律】**：凡是执行 `npm install`, `npm run build`, `cargo check`, `cargo build`, `tsc` 等耗时超过 5 秒的命令，**绝对禁止直接在终端输入！**

**必须且只能**使用全局脚本执行：
```bash
# ✅ 正确：使用 async_run.sh 脚本
./async_run.sh "npm install --registry=https://registry.npmmirror.com" "npm.log"
./async_run.sh "cargo check --manifest-path src-tauri/Cargo.toml" "cargo-check.log"
./async_run.sh "npx --yes tsc --noEmit" "tsc.log"

# ❌ 错误：直接执行会阻塞整个调度链
npm install
cargo check
npx --yes tsc --noEmit
```

**执行完毕后必须确认结果**：在下一个思考回合调用 `tail -n 50 <日志文件>` 确认结果，直到确认成功或失败才能汇报任务完成。

**异步执行脚本位置**：`./async_run.sh`（项目根目录）

**禁止行为**：
- ❌ 直接执行 `npm install`（可能卡死整个调度链）
- ❌ 直接执行 `cargo build`（首次编译可能需要 10+ 分钟）
- ❌ 使用阻塞式等待执行长耗时命令
- ❌ 使用 nohup 但不进行日志抽查（会导致逻辑断层）

### 3. npx 命令必须加 --yes 参数（防交互阻塞铁律）

**规则**：所有 `npx` 命令必须加 `--yes` 参数，跳过交互式确认提示：

```bash
# ✅ 正确
npx --yes tsc --noEmit
npx --yes prettier --write "src/**/*.{ts,svelte}"

# ❌ 错误（会弹出 y/n 提示导致终端死锁）
npx tsc --noEmit
npx prettier --write "src/**/*.{ts,svelte}"
```

### 4. 长耗时测试/构建命令必须后台执行

```bash
# 后台执行类型检查
nohup npx --yes tsc --noEmit > tsc-check.log 2>&1 &
sleep 5 && cat tsc-check.log  # 抽查日志

# 后台执行构建
nohup npm run build > build.log 2>&1 &
sleep 10 && tail -20 build.log  # 抽查日志

# 检查进程是否完成
ps aux | grep -E "tsc|vite" | grep -v grep
```

### 5. opencode.json 权限铁律（不可违反）

**规则**：opencode.json 中**绝对禁止**使用 `"deny"` 配置。所有权限只能配置为：
- `"allow"` — 自动允许通行（用于安全的默认操作）
- `"ask"` — 向人类询问（默认兜底，防止阻塞）

**标准权限模板**：
```json
{
  "permission": {
    "edit": {
      "*": "ask",
      "特定路径/**": "allow"
    },
    "bash": {
      "*": "ask",
      "特定命令*": "allow"
    },
    "task": {
      "*": "ask",
      "特定agent": "allow"
    }
  }
}
```

**禁止行为**：
- ❌ `"edit": "deny"` — 硬拒绝会导致 Agent 阻塞
- ❌ `"src/protocols/**": "deny"` — 特定路径拒绝会导致操作失败
- ✅ `"*": "ask"` — 通配符询问模式，永不阻塞

---

## UI 设计规范（Raycast Style）

> 详细规范请参见 `DESIGN.md` 文件。以下为核心要素摘要：

### 核心设计准则

1. **纯暗黑模式 (Single Dark Mode)**
   - 背景 Canvas: `#07080a`
   - 卡片 Surface 阶梯: `#0d0d0d` → `#101111` → `#121212`

2. **无阴影，仅边框 (No Shadows, Hairline Borders)**
   - 禁止使用任何 drop-shadow
   - 所有层级由 1px 极细边框（Hairline `#242728`）和背景色阶体现

3. **排版灵魂 (Typography ss03)**
   - 必须使用 Inter 字体
   - 全局启用 `font-feature-settings: "calt", "kern", "liga", "ss03"`
   - `ss03`（单层小写 g）是视觉签名

4. **主按钮 (White CTA)**
   - 所有 Primary Action 使用纯白背景 `#ffffff` 配纯黑文字 `#000000`

5. **圆角规范 (Border Radius)**
   - 微小元素: 4-6px
   - 标准按钮/卡片: 8-10px
   - 大型容器: 16px
   - 禁止使用 32px 以上

6. **点缀色克制 (Accent Colors)**
   - 彩色仅限用于图标、徽章或特定插图内部
   - 绝对不能用于大面积的 Chrome UI 或主按钮

### Tailwind 配置要求

Architect 在规划前端组件时，必须将以下 Token 映射到 `tailwind.config.js` 或全局 CSS：

```javascript
// tailwind.config.js 核心配置
module.exports = {
  theme: {
    extend: {
      colors: {
        canvas: '#07080a',
        surface: '#0d0d0d',
        'surface-elevated': '#101111',
        'surface-card': '#121212',
        hairline: '#242728',
        primary: '#ffffff',
        'on-primary': '#000000',
        ink: '#f4f4f6',
        body: '#cdcdcd',
        mute: '#9c9c9d',
        // ... 其他颜色
      },
      borderRadius: {
        xs: '4px',
        sm: '6px',
        md: '8px',
        lg: '10px',
        xl: '16px',
      },
      fontFamily: {
        sans: ['Inter', 'Inter Fallback', 'system-ui'],
      },
    },
  },
}
```

---

*最后更新: 2026-08-29 (v0.2.0 抓取规则外置引擎 + JD/天猫解析修复 + 双平台双架构构建矩阵)*
