// EGrab - Config Interface Protocol (L5)
// Derived from: docs/protocols/config-interface.md v1.0.0

import type { AppConfig, BrowserLaunchCommand } from './data-models';

/** 配置管理接口；Rust 模块路径为 src-tauri/src/config/。 */
export interface ConfigManager {
  /** 读取应用配置。 */
  get_config(): Promise<AppConfig>;

  /** 保存应用配置。 */
  set_config(config: AppConfig): Promise<boolean>;

  /** 获取浏览器启动命令参考。 */
  browser_launch_commands(): BrowserLaunchCommand[];
}
