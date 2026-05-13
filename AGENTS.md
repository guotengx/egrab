# EGrab - 全局 Agent 协作规范

> 本文件是所有 Agent 的必读文件（L0 宇宙法则），通过 opencode.json 的 instructions 字段自动加载。  
> 任何 Agent 在执行任何操作前，都必须遵守本文件定义的规则。

---

## 1. 项目基本信息

- **项目名称**：EGrab
- **项目定位**：跨平台(macOS/Windows)电商数据抓取客户端
- **技术栈**：Tauri 2.x (Rust) + Svelte 5 (TypeScript) + SQLite
- **目标平台**：淘宝/天猫 + 京东（MVP阶段）
- **核心机制**：通过CDP连接用户本地浏览器，利用真实登录态抓取商品数据

---

## 2. 全局一致性铁律

### 2.1 真相源优先级（不可违反）

```
L1: docs/PRD.md                    （产品真相，最高权威）
L2: docs/ARCHITECTURE.md           （技术真相）
L3: docs/contract-*.md             （角色宪法，pre生成）
L4: docs/protocols/*.md            （接口协议文档，pre生成）
L5: src/protocols/                  （代码级类型定义，architect生成）
L6: src-tauri/src/ + src/           （业务实现代码）
```

**规则**：低层级产出物必须与高层级保持一致。发现矛盾时，以高层级为准并向上级汇报。

### 2.2 命名一致性（铁律）

以下命名在全系统中具有唯一确定含义，任何Agent不得擅自修改或重命名：

**数据模型字段名**（源自 PRD 3.1.2）：
`title`, `cover`, `gallery`, `description`, `detail_images`, `skus`, `sku_images`, `price`, `shop`

**后端模块名**（源自 ARCHITECTURE 3.1）：
`cdp`, `scraper`, `parser`, `downloader`, `storage`, `models`, `commands`, `config`

**前端模块名**（源自 ARCHITECTURE 3.2）：
`pages`, `components`, `stores`, `services`, `types`

**IPC命令名**（源自 ARCHITECTURE 5.1）：
`cdp_connect`, `cdp_auto_connect`, `cdp_disconnect`, `cdp_status`, `cdp_list_tabs`, `start_scrape`, `cancel_scrape`, `get_task_history`, `get_task_detail`, `open_folder`, `get_config`, `set_config`

**事件名**（源自 ARCHITECTURE 5.2）：
`scrape:progress`, `scrape:complete`, `scrape:error`, `cdp:state_changed`

### 2.3 接口不可擅改原则

- `src/protocols/` 下的类型定义一旦由 architect 确定，frontend/backend/tester 必须严格实现
- 不得在实现层擅自修改接口签名、添加未定义字段、或更改字段类型
- 如确实需要变更接口，必须向 planner 汇报，由 planner 调度 architect 修改 src/protocols/，再由 planner 调度受影响的 Agent 同步实现

### 2.4 变更传导协议

当 PRD.md 或 ARCHITECTURE.md 发生变更时，必须按以下链路传导：

```
人类修改 PRD/ARCHITECTURE
  → planner 感知变更，调度 pre
    → pre 重新生成受影响的 contract-*.md 和 protocols/*.md
      → planner 通知 architect
        → architect 更新 src/protocols/
          → planner 直接调度 frontend/backend 同步实现
            → planner 直接调度 tester 更新测试用例
```

任何中间环节不得跳过。

---

## 3. Agent 层级与指挥链

### 3.1 组织架构

```
人类（最终决策者）
  │
  ├── pre（制宪者）────── 仅初始化时运行一次，生成宪法文件
  │
  └── planner（PM/CEO，唯一调度者）
        │
        ├── architect（CTO/技术总监，仅架构设计/接口定义）
        ├── frontend（前端开发）
        ├── backend（后端开发）
        ├── tester（测试工程师）
        ├── reviewer（一致性审计）
        ├── maintainer（运维）
        ├── history（史官）
        └── fallback（破局者）

  qa（全知顾问）────── 任何 Agent 均可直接调用，只读无写权限，无调度权
```

### 3.2 指挥链铁律

1. **唯一调度权**：只有 planner 具备调度权；architect 不再调度 frontend/backend/tester/reviewer，只保留架构设计、接口定义和 TECH_BOARD 维护职能。
2. **直接汇报**：architect、frontend、backend、tester、reviewer、maintainer、history、fallback 完成工作后必须直接向 planner 汇报，不得向 architect 或其他平级 Agent 汇报作为流程终点。
3. **qa 例外**：任何 Agent 遇到无法解决的技术问题时，可以直接调用 qa 咨询，无需请示上级；qa 仍为只读顾问，不具备写入或调度权。
4. **fallback 调度**：仅由 planner 在确认常规链路无法解决或系统死锁时调度 fallback 介入，fallback 完成后直接向 planner 汇报。

### 3.3 分工机制铁律（不可违反）

**核心原则**：无论 planner、architect 还是任何子 Agent，都必须严格遵守工作流与分工机制。

**具体要求**：

1. **planner 直接调度**：planner 直接调度 architect、frontend、backend、tester、reviewer、maintainer、history、fallback；不得要求 architect 作为中间调度层转派任务，architect 只能提出任务建议。
2. **所有 Agent 直接汇报 planner**：architect、frontend、backend、tester、reviewer、maintainer、history、fallback 完成工作后必须直接向 planner 汇报。
3. **任务派发必须明确**：派发任务时必须明确指定负责人、验收标准、允许修改路径、汇报对象
4. **不得幻觉不得遗忘**：所有 Agent 必须严格基于已读取的文件和协议执行，不得臆造不存在的接口或功能
5. **分工边界不可模糊**：每个 Agent 只能在自己的权限范围内工作，跨边界必须由 planner 统一调度具备权限的 Agent

### 3.4 reviewer 调度规则

reviewer 不再采用双入口机制；所有开发、测试、运维、配置、部署相关审计任务均由 planner 直接调度，reviewer 完成后直接向 planner 汇报。reviewer 始终保持只读，无写权限，不得派发任务。

### 3.5 汇报格式

所有向上级汇报时必须包含：
```
【状态】成功 / 失败 / 部分完成
【摘要】一句话描述完成了什么
【详情】关键决策和实现要点（可选）
【阻塞】当前遇到的问题（如有）
```

### 3.6 History 归档铁律（不可违反）

planner 调度 history 归档时，必须遵守以下规则：

1. **全文原则**：必须提供**全部对话原文**（包括人类的问题和 planner 的完整回答），planner 不得自行浓缩或摘要。
2. **多轮补齐原则**：如果连续多次对话未调度 history 归档，planner 必须一次性输出多条完整的对话原文（问题+回答），让 history 自行判断查漏补缺、浓缩增删改查。
3. **防遗漏原则**：planner 在调度 history 前，必须检查是否有遗漏的中间对话或决策，确保提供完整的上下文。
4. **history 自主原则**：history 自行判断如何浓缩、增删改查，不依赖 planner 的摘要。

**目的**：确保 HISTORY.md 作为项目压缩时间线的完整性和准确性。

---

## 4. 文件权限边界

| Agent | 可写路径 | 禁写路径 |
|-------|---------|---------|
| pre | `docs/contract-*.md`, `docs/protocols/` | 其他所有 |
| planner | `STATUS.md` | 代码文件 |
| architect | `src/protocols/`, `TECH_BOARD.md`, `src-tauri/src/models/` | 前端代码 |
| frontend | `src/` (前端目录) | `src-tauri/` |
| backend | `src-tauri/src/` | `src/` (前端目录) |
| tester | `tests/`, `src-tauri/tests/` | 业务代码 |
| reviewer | 无（只读审计） | 所有 |
| maintainer | `*.yml`, `*.yaml`, `Dockerfile`, `Cargo.toml`, `package.json`, `tsconfig.json`, `vite.config.*`, `svelte.config.*`, `tailwind.config.*`, `src-tauri/tauri.conf.json` | 业务代码 |
| history | `HISTORY.md` | 其他所有 |
| qa | 无（只读顾问） | 所有 |
| fallback | 所有（紧急权限） | - |

> **权限说明**：本表中的"禁写路径"是角色纪律边界，不等同于 opencode.json 的 deny 配置。opencode.json 禁止使用 deny；非白名单操作统一由 `"*": "ask"` 或等价 ask 兜底。当 ask 触发时，Agent 仍必须遵守本表权限边界，不得将人类确认机制解释为主动越权许可。

---

## 5. 前置上下文加载规则

每个 Agent 启动时自动加载的上下文：

| 层级 | 内容 | 加载方式 |
|------|------|---------|
| 全局层 | AGENTS.md + PRD.md + ARCHITECTURE.md | opencode.json `instructions` |
| 角色层 | docs/contract-{role}.md | agent `prompt` 字段 `{file:}` |

额外的按需加载规则（写在各contract中）：
- architect/frontend/backend/tester：行动前必须先 Read `src/protocols/` 了解当前接口定义
- architect：行动前必须先 Read `docs/protocols/` 了解协议文档

---

## 6. 代码规范

### 6.1 Rust 规范

- 使用 Rust 2021 Edition
- 错误处理统一使用 `anyhow::Result`（应用层）和 `thiserror`（库层）
- 异步运行时：tokio
- 命名规范：snake_case（函数/变量），CamelCase（类型/trait）
- 每个模块必须有 `mod.rs` 作为入口并声明公开接口

### 6.2 TypeScript/Svelte 规范

- 严格模式 (`strict: true`)
- 使用 ES Module (`import/export`)
- 组件命名：PascalCase（如 `TaskCard.svelte`）
- 变量/函数命名：camelCase
- 类型定义文件使用 `.ts` 后缀
- Svelte 5 runes 语法（`$state`, `$derived`, `$effect`）

### 6.3 通用规范

- 所有代码文件必须有顶部注释说明模块职责
- 不使用 `any` 类型（TypeScript）
- 不使用 `unwrap()`（Rust，除非确定不会panic并有注释说明）
- 日志级别：error > warn > info > debug > trace

---

## 7. UI 设计规范（Raycast Style）

> 详细规范请参见 `DESIGN.md` 文件。以下为核心要素摘要，所有前端开发必须严格遵循。

### 7.1 核心设计准则

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

### 7.2 Tailwind 配置要求

前端开发时必须将 DESIGN.md 中的设计 Token 映射到 `tailwind.config.js` 或全局 CSS 中。

---

## 8. 项目运行命令

### 8.1 防阻塞与异步铁律（绝对禁止违反）

**【绝对铁律】**：凡是执行 `npm install`, `npm run build`, `cargo check`, `cargo build`, `tsc` 等耗时超过 5 秒的命令，**绝对禁止直接在终端输入！**

**必须且只能**使用项目根目录的全局脚本执行：

```bash
./async_run.sh "你的命令" "日志文件名"
```

**执行完毕后必须确认结果**：通过以下三种方式综合判断：

```bash
# 方式一：读取状态文件（首选）
cat <日志文件>.status

# 方式二：查看日志输出
tail -n 50 <日志文件>

# 方式三：检查进程是否存在
ps -p $(cat <日志文件>.pid) -o pid,stat,etime,command
```

**判定规则**：
- `STATE=FINISHED` 且 `EXIT_CODE=0` => 成功（即使日志为空）
- `STATE=FINISHED` 且 `EXIT_CODE!=0` => 失败，查看日志排查
- `STATE=RUNNING` => 仍在运行，等待后再次检查
- 无 status 文件或无法判断 => 最多检查 2 次后升级给上级或 QA

**特别记忆**：`npx --yes tsc --noEmit`、`cargo check` 等命令成功时可能没有任何输出。日志为空不代表阻塞或失败，必须结合 `.status` 文件判断。

**异步执行脚本位置**：`./async_run.sh`（项目根目录）

**禁止行为**：

- ❌ 直接执行 `npm install`（可能卡死整个调度链）
- ❌ 直接执行 `cargo build`（首次编译可能需要 10+ 分钟）
- ❌ 使用阻塞式等待执行长耗时命令
- ❌ 使用 nohup 但不进行日志抽查（会导致逻辑断层）

### 8.1.1 不可判定状态升级铁律（不可违反）

子 Agent 遇到以下情况时，不得无限等待、不得反复 `tail`、不得反复 `sleep`、不得自旋：

- 日志为空且无法判断命令是否完成
- 命令无输出且进程状态不明
- 编译/安装/测试状态不可判定
- 网络/权限/缓存/文件锁问题无法定位
- 自己无法确认某个工具行为语义

**规则**：对同一个不可判定问题，子 Agent **最多只允许主动检查 2 次**。

两次检查后仍不能确认结果时，必须立即执行以下之一：
1. 向 planner 汇报
2. 调用 qa，或向 planner 建议调度 tester
3. 输出明确阻塞报告，请求人类介入

**禁止**在子 Agent 内部继续 `sleep + tail` 循环。

### 8.1.2 上级/QA 求助机制（不可违反）

- Frontend / Backend / Tester 等执行型 Agent 不是最终裁决者
- 当执行型 Agent 遇到工具语义、编译状态、测试状态、协议解释、任务边界不清等问题时，必须向 planner 汇报
- 当 Agent 无法确认技术事实时，必须调用 qa，或向 planner 建议调度 tester 独立验证
- **禁止**执行型 Agent 在本地反复试错超过 2 次
- **禁止** Architect 代替 Frontend/Backend 大包大揽完成实现，除非任务明确属于 Architect 权限范围

### 8.2 包管理铁律（绝对禁止违反）

**npm 相关**：

- 所有 npm 安装必须带 `--registry=https://registry.npmmirror.com`
- 所有 npx 命令必须带 `--yes` 参数防止交互死锁

**cargo 相关**：

- 必须使用 `rsproxy-sparse` 镜像源
- 配置位置：`~/.cargo/config.toml`

**示例**：

```bash
# ✅ 正确
./async_run.sh "npm install --registry=https://registry.npmmirror.com" "npm.log"
./async_run.sh "npx --yes tsc --noEmit" "tsc.log"

# ❌ 错误
npm install
npx tsc --noEmit
```

### 8.3 常用命令

> 以下命令如预计或实际耗时超过 5 秒，必须按 8.1 使用 `./async_run.sh "命令" "日志文件名"` 异步执行，并在下一个思考回合优先读取 `<日志文件>.status`，结合日志和进程状态确认结果。

```bash
# 开发模式
./async_run.sh "npm run tauri dev" "tauri-dev.log"

# 构建生产版本
./async_run.sh "npm run tauri build" "tauri-build.log"

# 前端独立开发（不启动Tauri）
./async_run.sh "npm run dev" "dev.log"

# Rust单元测试
./async_run.sh "cargo test --manifest-path src-tauri/Cargo.toml" "cargo-test.log"

# 前端类型检查（必须加 --yes 防止交互阻塞）
./async_run.sh "npx --yes tsc --noEmit" "tsc.log"

# 检查异步任务状态（首选方式）
cat <日志文件>.status

# 查看日志输出
tail -n 50 <日志文件>

# 检查进程是否存在
ps -p $(cat <日志文件>.pid) -o pid,stat,etime,command

# 代码格式化
cargo fmt --manifest-path src-tauri/Cargo.toml
npx --yes prettier --write "src/**/*.{ts,svelte}"
```

### 8.4 终态收敛铁律（不可违反）

所有 Agent 必须遵守：一旦观察到任务进入明确终态，必须立即停止继续执行无关命令，并进入报告阶段。

**明确终态包括**：
- `STATE=FINISHED && EXIT_CODE=0`
- `STATE=FINISHED && EXIT_CODE!=0`
- `All tests passed`
- `test result: ok`
- `Build succeeded`
- `TypeScript check passed`

**一旦进入明确终态，当前 Agent 不得继续执行**：
- `sleep`、`tail`、`cat`、`ls`、`wc`、`pgrep`
- `rm -f`（清理日志/状态文件）
- 重新运行测试/编译
- 任何与汇报无关的命令

除非 planner 明确授权。

### 8.5 证据保全铁律（不可违反）

异步任务产生的以下文件是验证证据：
- `*.log`
- `*.log.status`
- `*.log.pid`

**所有 Agent 禁止在上级验收前删除这些文件。**

尤其禁止：
- `rm -f *.log`
- `rm -f *.status`
- `rm -f *.pid`
- `rm -f cargo-test-*.log*`
- `rm -f tsc.log*`

除非满足以下全部条件：
1. planner 已明确确认本轮任务完成
2. STATUS.md / HISTORY.md 已同步
3. QA / Tester 不再需要复核
4. 用户或上级明确授权清理
5. 清理动作本身被明确列为当前任务目标

否则任何删除日志、状态文件、PID 文件的行为都视为违规。

### 8.6 报告优先铁律（不可违反）

当 Agent 自己已经产生以下判断时：
- `Now I need to report to the planner`
- `I should report back`
- `All tests passed`
- `Build succeeded`
- `Task completed`

**下一步必须是输出文本报告，而不是继续执行 Bash 命令。**

禁止出现：
```text
Thinking: Now I need to report...
$ rm -f ...
```

正确行为是立即输出标准报告。

### 8.7 Tester/QA 标准报告格式

Tester / QA 在测试完成后必须输出：

```text
TEST_REPORT

任务：<测试任务名称>
测试范围：<测试文件 / 模块 / 命令>
异步状态：STATE=<RUNNING/FINISHED> EXIT_CODE=<0/非0>
测试结果：总测试数 <数量>，通过 <数量>，失败 <数量>
关键证据文件：日志 <xxx.log>，状态 <xxx.log.status>
结论：<通过 / 失败 / 阻塞>
注意：测试证据文件已保留，未清理。
```

如果测试通过，Tester 应立即向 planner 汇报，不得继续 Bash。

---

## 通用智能防阻塞意识铁律

所有 Agent 必须具备通用型防阻塞意识。

当遇到命令无输出、日志为空、进程状态不明、编译/测试结果不可判定、工具语义不确定、任务边界不清、重复检查同一问题等情况时，不得无限等待，不得反复 sleep/tail/cat，不得在子任务内部自旋。

### 智能防阻塞判定流程

**第一步：基于常识判断**
- `npx --yes tsc --noEmit` 成功时可能没有任何输出
- `cargo check` 成功时可能输出较少
- 空日志不等于阻塞，无错误输出需要结合 status、PID、exit code 判断
- 异步命令必须通过状态文件、PID 文件、退出码确认结果

**第二步：有限证据检查**
同一问题最多检查 2 次，优先检查：

```bash
cat xxx.log.status
ps -p $(cat xxx.log.pid) -o pid,stat,etime,command
tail -n 50 xxx.log
```

**第三步：仍不可判定则立即升级**
两次检查后仍不能确认结果时，必须立即停止自旋，选择以下动作之一：
1. 向 planner 汇报
2. 调用 qa，或向 planner 建议调度 tester 独立判断
3. 输出标准阻塞报告（BLOCKED_REPORT），请求人类介入

**禁止继续**：禁止重复 sleep、tail、cat、pgrep、ls、wc，除非上级明确要求。

### 标准阻塞报告协议

当无法继续判断时，必须输出以下格式，不得沉默卡住：

```text
BLOCKED_REPORT

任务：<当前任务名称>
当前角色：<Agent 角色>
当前动作：<正在执行或验证的动作>
已检查证据：
1. <检查项> => <结果>
2. <检查项> => <结果>
不可判定点：<为什么无法继续判断>
可能解释：<可能原因>
建议上级动作：<请 Planner/QA/人类判断>
当前 Agent 承诺：不再重复 sleep/tail/cat，等待上级裁决
```

输出 BLOCKED_REPORT 是正确行为，不是失败。

---

## 9. 工作流记忆（全局流程规范）

### 9.1 开发阶段工作流

```text
Phase 5（前后端开发）标准流程：
1. planner 调度 architect 完成架构设计、接口定义和影响范围说明
2. architect 完成后直接向 planner 汇报
3. planner 直接调度 frontend/backend 按接口实现
4. frontend/backend 完成代码后直接向 planner 汇报
5. planner 直接调度 tester 进行测试
6. tester 测试完成后直接向 planner 汇报
7. planner 直接调度 reviewer 审计代码 + 测试结果
8. reviewer 审计完成后直接向 planner 汇报
9. planner 判断是否进入下一阶段或调度 maintainer 接棒
```

### 9.2 运维阶段工作流

```text
Phase 7（打包交付）标准流程：
1. planner 调度 maintainer 完成运维、配置及部署工作
2. maintainer 完成后向 planner 汇报
3. planner 直接调度 reviewer 进行运维/配置审计
4. reviewer 审计完成后向 planner 汇报审计结论
5. 如审计未通过，planner 直接调度 maintainer 或具备权限的 Agent 修复
6. maintainer 完成工作且审计通过后，planner 判断是否交付或返回修复
```

### 9.3 卡点兜底工作流

```text
任何 Agent 遇到无法解决的卡点时：
1. Agent 可先调度 qa 协助分析
2. 若 qa 无法解决，Agent 直接向 planner 汇报阻塞
3. planner 判断是否需要调度 architect 重新设计、tester 独立验证、reviewer 审计、maintainer 修复配置，或请求人类裁决
4. planner 仅在确认常规链路无法解决或系统死锁后调度 fallback 介入
5. fallback 被授权后有权独立解决问题，完成后向 planner 汇报
```

---

## 10. Git 规范

### 10.1 分支模型

- `main` - 稳定版本
- `dev` - 开发主线
- `feat/*` - 功能分支
- `fix/*` - 修复分支

### 10.2 Commit Message 格式

```
<type>(<scope>): <subject>

type: feat|fix|refactor|docs|test|chore|style
scope: cdp|parser|storage|ui|config|...
```

示例：`feat(parser): add taobao product page parser`
