// EGrab - Config Store
// Manages application configuration state using Svelte 5 runes
// Delegates to IPC service layer for persistence

import type { AppConfig } from '../protocols';
import { getConfig, setConfig } from '../services/ipc';

/** Extract human-readable error message from Tauri IpcError or standard Error. */
function extractErrorMessage(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    return String((err as { message: unknown }).message);
  }
  if (err instanceof Error) return err.message;
  return String(err);
}

/** Default configuration values */
const DEFAULT_CONFIG: AppConfig = {
  cdp_port: 9222,
  storage_root: '',
  image_concurrency: 3,
  browser_launch_commands: [],
};

/**
 * Reactive config store using Svelte 5 runes.
 * Provides loadConfig() and saveConfig() methods.
 */
function createConfigStore() {
  let config = $state<AppConfig>({ ...DEFAULT_CONFIG });
  let loading = $state<boolean>(false);
  let saving = $state<boolean>(false);
  let error = $state<string | null>(null);

  /** Load config from backend via IPC. */
  async function loadConfig(): Promise<void> {
    loading = true;
    error = null;
    try {
      config = await getConfig();
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `加载配置失败: ${message}`;
      // Keep default config on error
    } finally {
      loading = false;
    }
  }

  /** Save config to backend via IPC. */
  async function saveConfig(newConfig: AppConfig): Promise<boolean> {
    saving = true;
    error = null;
    try {
      const success = await setConfig(newConfig);
      if (success) {
        config = { ...newConfig };
      }
      return success;
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `保存配置失败: ${message}`;
      return false;
    } finally {
      saving = false;
    }
  }

  return {
    get config(): AppConfig {
      return config;
    },
    get loading(): boolean {
      return loading;
    },
    get saving(): boolean {
      return saving;
    },
    get error(): string | null {
      return error;
    },
    loadConfig,
    saveConfig,
  };
}

export const configStore = createConfigStore();
