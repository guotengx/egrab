<!-- EGrab - Home Page -->
<!-- Main page with auto CDP connect, URL input, and task history overview -->
<!-- Design: Raycast-inspired dark theme, surface ladder cards -->

<script lang="ts">
  import UrlInput from '../components/UrlInput.svelte';
  import { cdpNavigate } from '../services/ipc';
  import { configStore } from '../stores/config.svelte';
  import { connectionStore } from '../stores/connection.svelte';
  import { tasksStore } from '../stores/tasks.svelte';

  interface Props {
    onNavigate?: (page: string) => void;
  }

  let { onNavigate }: Props = $props();

  let config = $derived(configStore.config);
  let connectionState = $derived(connectionStore.state);
  let isScraping = $derived(tasksStore.loading);
  let scrapeError = $derived(tasksStore.error);
  let isConnecting = $derived(connectionStore.isOperating);
  let connectionError = $derived(connectionStore.error);

  // Load config and auto-connect on mount
  $effect(() => {
    configStore.loadConfig();
    connectionStore.autoConnect();
  });

  /** Handle scrape submission from UrlInput: start scrape via tasksStore and navigate to progress page. */
  async function handleScrapeSubmit(url: string, _platform: string, force: boolean = false): Promise<void> {
    await tasksStore.startScrape(url, force);
    // Only navigate if startScrape succeeded (currentTask is set)
    if (tasksStore.currentTask && onNavigate) {
      onNavigate('progress');
    }
    // If error occurred, stay on home page — error will be displayed via scrapeError
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <main class="flex-1 p-6 overflow-auto">
    <div class="max-w-3xl mx-auto space-y-6">
      <!-- Hero Section -->
      <section class="text-center pt-4 pb-2">
        <h1 class="text-2xl font-medium text-ink tracking-tight mb-2">EGrab</h1>
        <p class="text-body text-sm">电商数据抓取客户端</p>
      </section>

      <!-- Auto CDP Connection Status -->
      <section class="bg-surface border border-hairline rounded-lg p-6">
        <h2 class="text-sm font-medium text-on-dark mb-4">浏览器连接</h2>

        <!-- Status Indicator Row -->
        <div class="flex items-center gap-3">
          <span
            class="inline-block w-2 h-2 rounded-full {connectionState.type === 'Connected'
              ? 'bg-accent-green'
              : connectionState.type === 'Connecting' || connectionState.type === 'Reconnecting'
                ? 'bg-accent-yellow'
                : 'bg-accent-red'}"
          ></span>
          <span class="text-body text-sm">
            {#if connectionState.type === 'Connected'}
              已连接 (浏览器: {connectionState.browser_version})
            {:else if connectionState.type === 'Connecting'}
              正在自动连接浏览器...
            {:else if connectionState.type === 'Reconnecting'}
              重连中 (第 {connectionState.attempt} 次)
            {:else if connectionState.type === 'Failed'}
              连接失败: {connectionState.reason}
            {:else}
              未连接
            {/if}
          </span>
          {#if isConnecting}
            <svg class="animate-spin h-4 w-4 text-body" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
          {/if}
        </div>

        <!-- Error Message -->
        {#if connectionError}
          <div class="mt-3 bg-surface-elevated border border-hairline rounded-md p-3">
            <p class="text-accent-red text-xs">{connectionError}</p>
          </div>
        {/if}

        <!-- Login Hint -->
        {#if connectionState.type === 'Connected'}
          <div class="mt-3 bg-surface-elevated border border-hairline rounded-md p-4 space-y-3">
            <p class="text-body text-xs">请在浏览器中登录以下平台，然后粘贴商品链接开始抓取</p>
            <div class="flex items-center gap-3">
              <button
                type="button"
                onclick={() => cdpNavigate('https://passport.jd.com')}
                class="bg-surface-card text-on-dark border border-hairline rounded-md font-medium px-4 py-2 text-xs transition-colors cursor-pointer hover:border-hairline-strong hover:text-ink"
              >
                京东登录
              </button>
              <button
                type="button"
                onclick={() => cdpNavigate('https://login.taobao.com')}
                class="bg-surface-card text-on-dark border border-hairline rounded-md font-medium px-4 py-2 text-xs transition-colors cursor-pointer hover:border-hairline-strong hover:text-ink"
              >
                淘宝/天猫登录
              </button>
            </div>
          </div>
        {/if}
      </section>

      <!-- URL Input Section -->
      <section class="bg-surface border border-hairline rounded-lg p-6">
        <h2 class="text-sm font-medium text-on-dark mb-4">商品抓取</h2>
        <UrlInput onSubmit={handleScrapeSubmit} loading={isScraping} />

        <!-- Error Message -->
        {#if scrapeError}
          <div class="bg-surface border border-hairline rounded-lg p-4 mt-3">
            <p class="text-accent-red text-sm">{scrapeError}</p>
          </div>
        {/if}
      </section>

      <!-- Config Summary Card -->
      <section class="bg-surface border border-hairline rounded-lg p-6">
        <h2 class="text-sm font-medium text-on-dark mb-4">当前配置</h2>
        {#if configStore.loading}
          <p class="text-mute text-sm">加载中...</p>
        {:else if configStore.error}
          <p class="text-accent-red text-sm">{configStore.error}</p>
        {:else}
          <dl class="space-y-3 text-sm">
            <div class="flex justify-between items-center">
              <dt class="text-mute">CDP 端口</dt>
              <dd class="text-on-dark font-mono text-xs">{config.cdp_port}</dd>
            </div>
            <div class="flex justify-between items-center">
              <dt class="text-mute">存储根目录</dt>
              <dd class="text-on-dark font-mono text-xs">{config.storage_root || '(默认)'}</dd>
            </div>
            <div class="flex justify-between items-center">
              <dt class="text-mute">图片并发数</dt>
              <dd class="text-on-dark font-mono text-xs">{config.image_concurrency}</dd>
            </div>
          </dl>
        {/if}
      </section>
    </div>
  </main>
</div>
