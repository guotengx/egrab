# EGrab - 代码级接口定义

> 本目录下的所有文件由 **architect agent** 生成，是 L5 级别的代码级类型定义。

---

## 目录说明

本目录存放 TypeScript 类型定义文件，供前后端共同引用。

architect 根据 `docs/protocols/*.md`（pre生成的接口协议文档）来编写此目录下的 `.ts` 文件。

---

## 已生成文件

| 文件 | 内容 |
|------|------|
| `data-models.ts` | 全部核心数据模型：ProductData, ImageRef, SkuItem, PriceRange, ShopInfo, Description, SpecItem, Task, ImageRecord, TaskFilter, TaskSummary, TaskDetail, TaskResult, ScrapeErrorInfo, IpcError, ConnectionInfo, ConnectionState, TabInfo, AppConfig, MetaJsonDocument, RawJsonDocument 等 |
| `ipc-commands.ts` | IPC 命令 TypeScript 接口定义（参数类型 + 返回值类型 + IpcResult + IpcCommand 联合类型） |
| `events.ts` | 后端事件 payload TypeScript 类型定义（scrape:progress, scrape:complete, scrape:error, cdp:state_changed） |
| `parser.ts` | PlatformParser 接口、PageHandle、PageContext、ParserConfig、ParseResult |
| `storage.ts` | StorageEngine 接口、TaskUpdate、ImageIndexInput、DuplicateTaskConflict、ArchiveStructure、ArchiveEntry |
| `index.ts` | 统一导出所有类型 |

---

## 一致性链条

```
L1: docs/PRD.md (产品真相)
  ↓
L2: docs/ARCHITECTURE.md (技术真相)
  ↓
L4: docs/protocols/*.md (接口协议文档，pre生成)
  ↓
L5: src/protocols/*.ts (本目录，architect生成)  ← 你在这里
  ↓
L6: src-tauri/src/ + src/ (业务实现代码)
```

---

## 使用规则

- frontend agent：编码前必须先 Read 本目录了解接口类型
- backend agent：编码前必须先 Read 本目录了解接口类型，Rust实现必须与此对齐
- tester agent：测试用例必须基于本目录的类型定义编写
- **任何Agent不得擅自修改本目录文件**，修改权限仅限 architect
