# EGrab - 接口协议文档目录

> 本目录下的所有文件由 **pre agent** 生成，是 L4 级别的接口协议定义。

---

## 文件清单（已生成）

| 文件 | 内容 | 状态 |
|------|------|------|
| `data-models.md` | 核心数据模型协议（ProductData, ImageRef, SkuItem等） | 已生成 |
| `ipc-commands.md` | IPC命令接口协议（Tauri Commands完整签名） | 已生成 |
| `events.md` | 事件协议（Tauri Events的payload结构） | 已生成 |
| `parser-interface.md` | 平台解析器接口协议（PlatformParser trait） | 已生成 |
| `storage-interface.md` | 存储引擎接口协议（SQLite schema + 文件系统规范） | 已生成 |
| `cdp-manager-interface.md` | CDP 管理器接口协议（端口扫描、连接状态、页面导航与脚本执行） | 已生成 |
| `downloader-interface.md` | 图片下载器接口协议（批量下载、并发、失败降级与结果索引） | 已生成 |
| `scraper-engine-interface.md` | 抓取引擎接口协议（任务启动、取消、流程编排与解析输出） | 已生成 |
| `config-interface.md` | 配置管理接口协议（应用配置、浏览器启动命令与配置约束） | 已生成 |

> 自审日期：2026-05-09。上述状态以当前 `docs/protocols/` 目录实际文件为准。

---

## 使用说明

- **architect** 在设计 `src/protocols/` 代码级类型定义时，必须参照本目录下的协议文档
- **frontend/backend** 在编码前必须先 Read `src/protocols/`，而 `src/protocols/` 又源自本目录
- 一致性链条：`docs/protocols/*.md` → `src/protocols/` → 业务代码
- `start_scrape` 的协议参数为 `url: string` 与可选 `force?: boolean`；`force` 未提供时必须按 `false` 处理。

---

## 一致性要求

本目录下的所有定义必须与以下真相源保持一致：

1. `docs/PRD.md` 3.1.2节（数据模型字段名）
2. `docs/ARCHITECTURE.md` 第4节（数据模型定义）、第5节（IPC接口定义）
3. `AGENTS.md` 2.2节（全局命名一致性）
