// EGrab - CDP Manager Interface Protocol (L5)
// Derived from: docs/protocols/cdp-manager-interface.md v1.0.0

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
