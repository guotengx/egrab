# Protocol: CDP Manager Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-09
- 依赖的真相源：`docs/PRD.md` 1.0.0 第3.2节、`docs/ARCHITECTURE.md` 1.0.0 第3.1/4.2/5.1节、`docs/protocols/data-models.md`

## 类型定义

```ts
import type { ConnectionInfo, ConnectionState, JsonValue, TabInfo } from './data-models';

/** CDP 管理器接口；Rust 模块路径为 src-tauri/src/cdp/。 */
export interface CdpManager {
  /** 扫描本地 CDP 端口；MVP 默认至少检查 9222。 */
  scan_ports(candidates?: number[]): Promise<CdpEndpoint[]>;

  /** 连接 127.0.0.1:{port}，超时 10s。 */
  connect(port: number): Promise<ConnectionInfo>;

  /** 主动断开当前 CDP 连接。 */
  disconnect(): Promise<boolean>;

  /** 返回当前连接状态。 */
  status(): Promise<ConnectionState>;

  /** 列出当前浏览器标签页。 */
  list_tabs(): Promise<TabInfo[]>;

  /** 导航到 URL 并等待页面加载完成。 */
  navigate(url: string): Promise<void>;

  /** 在当前页面执行 JavaScript，返回 serde_json::Value 语义的 JSON 值。 */
  evaluate(script: string): Promise<JsonValue>;
}

/** 扫描到的 CDP endpoint。 */
export interface CdpEndpoint {
  /** CDP 端口，u16 范围 1-65535。 */
  port: number;
  /** WebSocket endpoint，例如 ws://127.0.0.1:9222/devtools/browser/...。 */
  endpoint: string;
  /** 浏览器版本；未知时为 null。 */
  browser_version: string | null;
}
```

## 约束
- CDP 连接仅允许 `127.0.0.1`，不得连接局域网、远程主机或用户输入的任意 host。
- `connect(port)` 必须先进入 `Connecting`，成功后进入 `Connected`，失败后进入 `Failed` 并发出 `cdp:state_changed`。
- 断线自动重连最多 3 次，间隔 2s；重连状态使用 `ConnectionState` 的 `Reconnecting.attempt`，范围 1-3。
- 连接超时固定为 10s；页面加载超时按 ARCHITECTURE 错误策略为 30s。
- 自动扫描本地 CDP 端口是 `cdp` 模块内部行为，不新增 IPC 命令；前端仍通过 `cdp_connect`、`cdp_status`、`cdp_list_tabs` 等既有 IPC 交互。

## 示例

```json
{
  "scan_ports": [{ "port": 9222, "endpoint": "ws://127.0.0.1:9222/devtools/browser/abc", "browser_version": "Chrome/124.0.0.0" }],
  "state_changed": { "type": "Connected", "browser_version": "Chrome/124.0.0.0" }
}
```
