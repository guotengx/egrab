# Protocol: Config Interface

## 版本
- 版本号：1.0.0
- 创建日期：2026-05-09
- 依赖的真相源：`docs/PRD.md` 1.0.0 第3.4.4节、`docs/ARCHITECTURE.md` 1.0.0 第3.1/5.1/8节、`docs/protocols/data-models.md`

## 类型定义

```ts
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
```

## 约束
- `cdp_port` 默认 9222，范围 1-65535。
- `storage_root` 默认 macOS `~/EGrab/`，Windows `%USERPROFILE%\EGrab\`。
- `image_concurrency` 默认 3，最小 1，最大 10。
- 浏览器启动命令参考来自 ARCHITECTURE 第8节：macOS Chrome `open -a "Google Chrome" --args --remote-debugging-port=9222`；Windows Chrome `chrome.exe --remote-debugging-port=9222`；Edge 可按相同端口参数生成。
- 配置中不得存储账号密码、Cookie、遥测标识或远程服务器地址。

## 示例

```json
{
  "cdp_port": 9222,
  "storage_root": "~/EGrab/",
  "image_concurrency": 3,
  "browser_launch_commands": [
    { "os": "macos", "browser": "chrome", "command": "open -a \"Google Chrome\" --args --remote-debugging-port=9222" },
    { "os": "windows", "browser": "chrome", "command": "chrome.exe --remote-debugging-port=9222" }
  ]
}
```
