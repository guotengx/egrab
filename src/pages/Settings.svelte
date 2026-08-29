<!-- EGrab - Settings Page -->
<!-- Configuration editing form with save functionality -->
<!-- Design: Raycast-inspired dark theme, surface-ladder form cards -->

<script lang="ts">
  import type { AppConfig } from '../protocols';
  import { configStore } from '../stores/config.svelte';
  import {
    getRulesInfo,
    reloadRules,
    openRulesFolder,
    dumpPageSnapshot,
    type RulesInfo,
  } from '../services/ipc';

  // Local form state bound to inputs
  let cdpPort = $state<number>(9222);
  let storageRoot = $state<string>('');
  let imageConcurrency = $state<number>(3);
  let saveMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);
  let isSaving = $state<boolean>(false);

  // Sync form state when config loads
  $effect(() => {
    const config = configStore.config;
    cdpPort = config.cdp_port;
    storageRoot = config.storage_root;
    imageConcurrency = config.image_concurrency;
  });

  /** Validate and save configuration. */
  async function handleSave(): Promise<void> {
    // Basic validation
    if (cdpPort < 1 || cdpPort > 65535) {
      saveMessage = { type: 'error', text: 'CDP 端口必须在 1-65535 范围内' };
      return;
    }
    if (imageConcurrency < 1 || imageConcurrency > 10) {
      saveMessage = { type: 'error', text: '图片下载并发数必须在 1-10 范围内' };
      return;
    }

    isSaving = true;
    saveMessage = null;

    const newConfig: AppConfig = {
      cdp_port: cdpPort,
      storage_root: storageRoot,
      image_concurrency: imageConcurrency,
      browser_launch_commands: configStore.config.browser_launch_commands,
    };

    const success = await configStore.saveConfig(newConfig);
    isSaving = false;

    if (success) {
      saveMessage = { type: 'success', text: '配置已保存' };
    } else {
      saveMessage = { type: 'error', text: configStore.error ?? '保存失败' };
    }

    // Auto-clear message after 3 seconds
    setTimeout(() => {
      saveMessage = null;
    }, 3000);
  }

  // ── 抓取规则包 ───────────────────────────────────────────────
  let rulesInfo = $state<RulesInfo | null>(null);
  let rulesMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);
  let rulesBusy = $state<boolean>(false);

  $effect(() => {
    void loadRulesInfo();
  });

  async function loadRulesInfo(): Promise<void> {
    try {
      rulesInfo = await getRulesInfo();
    } catch (err) {
      rulesInfo = null;
      rulesMessage = { type: 'error', text: `读取规则失败：${String(err)}` };
    }
  }

  function flashRulesMessage(type: 'success' | 'error', text: string): void {
    rulesMessage = { type, text };
    setTimeout(() => {
      rulesMessage = null;
    }, 6000);
  }

  async function handleOpenRules(): Promise<void> {
    try {
      await openRulesFolder();
    } catch (err) {
      flashRulesMessage('error', `打开规则目录失败：${String(err)}`);
    }
  }

  async function handleReloadRules(): Promise<void> {
    rulesBusy = true;
    try {
      rulesInfo = await reloadRules();
      flashRulesMessage('success', `规则校验通过（v${rulesInfo.version}）`);
    } catch (err) {
      flashRulesMessage('error', `规则校验失败：${String(err)}`);
    } finally {
      rulesBusy = false;
    }
  }

  async function handleDumpSnapshot(): Promise<void> {
    rulesBusy = true;
    try {
      const path = await dumpPageSnapshot();
      flashRulesMessage('success', `快照已导出：${path}`);
    } catch (err) {
      flashRulesMessage('error', `导出快照失败（请先连接浏览器并打开商品页）：${String(err)}`);
    } finally {
      rulesBusy = false;
    }
  }

  /** Determine OS for displaying correct launch command. */
  function getOsLabel(): string {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('mac')) return 'macOS';
    if (platform.includes('win')) return 'Windows';
    return 'Unknown';
  }

  /** Get the browser launch command for current platform. */
  function getLaunchCommand(): string {
    const commands = configStore.config.browser_launch_commands;
    const os = getOsLabel().toLowerCase() as 'macos' | 'windows';
    const cmd = commands.find((c) => c.os === os && c.browser === 'chrome');
    return cmd?.command ?? '';
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <main class="flex-1 p-6 overflow-auto">
    <div class="max-w-2xl mx-auto space-y-6">
      <h1 class="text-xl font-medium text-ink tracking-tight">设置</h1>

      {#if configStore.loading}
        <p class="text-mute text-sm">正在加载配置...</p>
      {:else}
        <form onsubmit={(e) => { e.preventDefault(); handleSave(); }} class="space-y-5">
          <!-- CDP Port Field -->
          <div class="bg-surface border border-hairline rounded-lg p-6">
            <label for="cdp-port" class="block text-sm font-medium text-on-dark mb-2">
              CDP 端口
            </label>
            <input
              id="cdp-port"
              type="number"
              bind:value={cdpPort}
              min={1}
              max={65535}
              class="w-full px-3 py-2 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm focus:border-hairline-strong focus:outline-none transition-colors"
            />
            <p class="mt-2 text-xs text-mute">
              Chrome/Edge 远程调试端口，默认 9222
            </p>
          </div>

          <!-- Storage Root Field -->
          <div class="bg-surface border border-hairline rounded-lg p-6">
            <label for="storage-root" class="block text-sm font-medium text-on-dark mb-2">
              存储根目录
            </label>
            <input
              id="storage-root"
              type="text"
              bind:value={storageRoot}
              placeholder="留空使用默认路径"
              class="w-full px-3 py-2 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm placeholder:text-stone focus:border-hairline-strong focus:outline-none transition-colors"
            />
            <p class="mt-2 text-xs text-mute">
              抓取数据的本地存储根目录
            </p>
          </div>

          <!-- Image Concurrency Field -->
          <div class="bg-surface border border-hairline rounded-lg p-6">
            <label for="image-concurrency" class="block text-sm font-medium text-on-dark mb-2">
              图片下载并发数
            </label>
            <input
              id="image-concurrency"
              type="number"
              bind:value={imageConcurrency}
              min={1}
              max={10}
              class="w-full px-3 py-2 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm focus:border-hairline-strong focus:outline-none transition-colors"
            />
            <p class="mt-2 text-xs text-mute">
              并发下载数量（1-10），默认 3
            </p>
          </div>

          <!-- Browser Launch Command Reference -->
          <div class="bg-surface border border-hairline rounded-lg p-6">
            <h3 class="text-sm font-medium text-on-dark mb-2">
              浏览器启动命令参考 ({getOsLabel()})
            </h3>
            {#if getLaunchCommand()}
              <pre class="bg-surface-elevated border border-hairline rounded-md p-3 text-xs text-charcoal overflow-x-auto whitespace-pre-wrap">{getLaunchCommand()}</pre>
            {:else}
              <p class="text-sm text-mute">暂无启动命令配置</p>
            {/if}
            <p class="mt-2 text-xs text-mute">
              请以该参数启动浏览器以启用 CDP 远程调试
            </p>
          </div>

          <!-- Save Button & Message -->
          <div class="flex items-center gap-4">
            <button
              type="submit"
              disabled={isSaving}
              class="bg-primary text-on-primary rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer disabled:bg-surface-elevated disabled:text-ash disabled:cursor-not-allowed hover:bg-primary-pressed"
            >
              {isSaving ? '保存中...' : '保存配置'}
            </button>

            {#if saveMessage}
              <span
                class:text-accent-green={saveMessage.type === 'success'}
                class:text-accent-red={saveMessage.type === 'error'}
                class="text-sm"
              >
                {saveMessage.text}
              </span>
            {/if}
          </div>
        </form>

        <!-- 抓取规则包 -->
        <div class="bg-surface border border-hairline rounded-lg p-6 space-y-4">
          <div>
            <h3 class="text-sm font-medium text-on-dark">抓取规则</h3>
            <p class="mt-2 text-xs text-mute leading-relaxed">
              平台改版导致抓不到数据时，直接编辑规则目录里的
              <code class="text-charcoal">*.extract.js</code>，保存后下一次抓取立即生效，
              <strong class="text-on-dark">无需重新编译或重装程序</strong>。
            </p>
          </div>

          {#if rulesInfo}
            <div class="bg-surface-elevated border border-hairline rounded-md p-3 space-y-1">
              <p class="text-xs text-mute">
                版本 <span class="text-on-dark">v{rulesInfo.version}</span>
                · 来源 <span class="text-on-dark">{rulesInfo.source === 'disk' ? '磁盘（可编辑）' : '内置兜底'}</span>
                · 平台 <span class="text-on-dark">{rulesInfo.platforms.map((p) => p.id).join(', ')}</span>
              </p>
              <p class="text-xs text-stone break-all">{rulesInfo.rules_dir}</p>
              {#if rulesInfo.error}
                <p class="text-xs text-accent-red">规则文件有语法错误，已回退内置规则：{rulesInfo.error}</p>
              {/if}
            </div>
          {:else}
            <p class="text-xs text-mute">正在读取规则信息...</p>
          {/if}

          <div class="flex flex-wrap items-center gap-3">
            <button
              type="button"
              onclick={handleOpenRules}
              class="bg-surface-elevated border border-hairline text-on-dark rounded-md px-4 py-2 text-sm transition-colors cursor-pointer hover:border-hairline-strong"
            >
              打开规则目录
            </button>
            <button
              type="button"
              onclick={handleReloadRules}
              disabled={rulesBusy}
              class="bg-surface-elevated border border-hairline text-on-dark rounded-md px-4 py-2 text-sm transition-colors cursor-pointer hover:border-hairline-strong disabled:text-ash disabled:cursor-not-allowed"
            >
              校验规则
            </button>
            <button
              type="button"
              onclick={handleDumpSnapshot}
              disabled={rulesBusy}
              class="bg-surface-elevated border border-hairline text-on-dark rounded-md px-4 py-2 text-sm transition-colors cursor-pointer hover:border-hairline-strong disabled:text-ash disabled:cursor-not-allowed"
            >
              导出页面快照
            </button>
          </div>

          {#if rulesMessage}
            <p
              class:text-accent-green={rulesMessage.type === 'success'}
              class:text-accent-red={rulesMessage.type === 'error'}
              class="text-xs break-all"
            >
              {rulesMessage.text}
            </p>
          {/if}

          <p class="text-xs text-stone leading-relaxed">
            「导出页面快照」会把浏览器当前页面的完整 DOM 和候选数据源存到规则目录下的
            <code>snapshots/</code>，用于快速定位平台改动。
          </p>
        </div>
      {/if}
    </div>
  </main>
</div>
