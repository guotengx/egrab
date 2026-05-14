<!-- EGrab - Archive Page -->
<!-- Browse scraped task history with search, filters, and task detail view -->
<!-- Design: Raycast dark theme, max-w-3xl layout, TaskCard list -->

<script lang="ts">
  import TaskCard from '../components/TaskCard.svelte';
  import { tasksStore } from '../stores/tasks.svelte';
  import type { Platform, TaskStatus } from '../protocols';
  import { resizeImages as ipcResizeImages } from '../services/ipc';

  interface Props {
    onNavigate?: (page: string) => void;
  }

  let { onNavigate }: Props = $props();

  // --- Local Filter State ---
  let searchKeyword = $state<string>('');
  let platformFilter = $state<Platform | ''>('');
  let statusFilter = $state<TaskStatus | ''>('');

  // --- Derived from store ---
  let taskHistory = $derived(tasksStore.taskHistory);
  let loading = $derived(tasksStore.loading);
  let taskError = $derived(tasksStore.error);
  let taskDetail = $derived(tasksStore.taskDetail);

  // --- Selected task ID for detail view ---
  let selectedTaskId = $state<string | null>(null);

  /** Platform filter options */
  const PLATFORM_OPTIONS: { value: Platform | ''; label: string }[] = [
    { value: '', label: '全部平台' },
    { value: 'taobao', label: '淘宝' },
    { value: 'tmall', label: '天猫' },
    { value: 'jd', label: '京东' },
  ];

  /** Status filter options */
  const STATUS_OPTIONS: { value: TaskStatus | ''; label: string }[] = [
    { value: '', label: '全部状态' },
    { value: 'success', label: '成功' },
    { value: 'failed', label: '失败' },
    { value: 'partial', label: '部分完成' },
  ];

  /** Apply filters and load history from backend. */
  async function applyFilters(): Promise<void> {
    const filter: Record<string, string | undefined> = {};
    if (searchKeyword.trim()) filter.keyword = searchKeyword.trim();
    if (platformFilter) filter.platform = platformFilter;
    if (statusFilter) filter.status = statusFilter;

    await tasksStore.loadHistory(filter as Parameters<typeof tasksStore.loadHistory>[0]);
  }

  /** Load history on mount and when filters change. */
  $effect(() => {
    applyFilters();
  });

  /** Handle task card click: load detail for the task. */
  async function handleTaskClick(taskId: string): Promise<void> {
    if (selectedTaskId === taskId) {
      // Toggle off
      selectedTaskId = null;
      tasksStore.clearTaskDetail();
    } else {
      selectedTaskId = taskId;
      await tasksStore.loadDetail(taskId);
    }
  }

  /** Handle open folder action from TaskCard. */
  async function handleOpenFolder(path: string | undefined): Promise<void> {
    if (path) {
      await tasksStore.openFolder(path);
    }
  }

  /** Handle delete action from TaskCard.
   *  Note: window.confirm() is unreliable in Tauri WebView;
   *  delete executes directly without confirmation dialog. */
  async function handleDeleteTask(taskId: string): Promise<void> {
    const success = await tasksStore.deleteTask(taskId);
    if (success) {
      // If the deleted task was selected, clear selection
      if (selectedTaskId === taskId) {
        selectedTaskId = null;
        tasksStore.clearTaskDetail();
      }
    }
  }

  /** Format price range for display. */
  function formatPrice(price: { min_price: number; max_price: number; currency: string }): string {
    if (price.min_price === price.max_price) {
      return `${price.currency} ${price.min_price.toFixed(2)}`;
    }
    return `${price.currency} ${price.min_price.toFixed(2)} - ${price.max_price.toFixed(2)}`;
  }

  /**
   * Svelte action that attaches event listeners for custom events.
   * Used to delegate TaskCard's CustomEvent dispatches to parent handlers.
   */
  function delegateEvents(node: HTMLElement, handlers: Record<string, (e: Event) => void>) {
    Object.entries(handlers).forEach(([event, handler]) => {
      node.addEventListener(event, handler);
    });
    return {
      destroy() {
        Object.entries(handlers).forEach(([event, handler]) => {
          node.removeEventListener(event, handler);
        });
      },
    };
  }

  /** Format ISO timestamp to locale-friendly string. */
  function formatTime(isoString: string): string {
    try {
      const date = new Date(isoString);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return isoString;
    }
  }

  // --- Resize Images State & Handler ---
  let resizeResult = $state<{
    total: number;
    resized: number;
    skipped: number;
    failed: number;
  } | null>(null);
  let resizeLoading = $state(false);
  let resizeError = $state<string | null>(null);

  async function handleResizeImages(taskId: string): Promise<void> {
    resizeLoading = true;
    resizeError = null;
    resizeResult = null;
    try {
      const result = await ipcResizeImages(taskId);
      resizeResult = {
        total: result.total,
        resized: result.resized,
        skipped: result.skipped,
        failed: result.failed,
      };
    } catch (err) {
      resizeError = err instanceof Error ? err.message : String(err);
    } finally {
      resizeLoading = false;
    }
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <main class="flex-1 p-6 overflow-auto">
    <div class="max-w-3xl mx-auto space-y-6">
      <!-- Header -->
      <section class="pt-4 pb-2">
        <h1 class="text-xl font-medium text-ink tracking-tight mb-1">存档浏览</h1>
        <p class="text-mute text-[13px]">查看已抓取的商品数据</p>
      </section>

      <!-- Search & Filters Bar -->
      <section class="bg-surface border border-hairline rounded-lg p-4 space-y-3">
        <!-- Search Input -->
        <div class="relative">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" class="absolute left-3 top-1/2 -translate-y-1/2 text-stone pointer-events-none">
            <circle cx="6" cy="6" r="4.5"/>
            <path d="M9.5 9.5L13 13" stroke-linecap="round"/>
          </svg>
          <input
            type="text"
            bind:value={searchKeyword}
            placeholder="搜索商品标题..."
            class="w-full h-9 pl-9 pr-3 py-2 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm placeholder:text-stone focus:border-hairline-strong focus:outline-none transition-colors"
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                applyFilters();
              }
            }}
          />
        </div>

        <!-- Filter Row: Platform + Status -->
        <div class="flex items-center gap-3">
          <!-- Platform Dropdown -->
          <select
            bind:value={platformFilter}
            onchange={() => applyFilters()}
            class="flex-1 h-8 px-3 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm focus:border-hairline-strong focus:outline-none transition-colors cursor-pointer"
          >
            {#each PLATFORM_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>

          <!-- Status Dropdown -->
          <select
            bind:value={statusFilter}
            onchange={() => applyFilters()}
            class="flex-1 h-8 px-3 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm focus:border-hairline-strong focus:outline-none transition-colors cursor-pointer"
          >
            {#each STATUS_OPTIONS as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
      </section>

      <!-- Task List or Empty State -->
      <section class="space-y-3">
        {#if loading && taskHistory.length === 0}
          <!-- Loading State -->
          <div class="flex flex-col items-center justify-center py-16 px-6">
            <p class="text-mute text-sm">加载中...</p>
          </div>

        {:else if taskHistory.length === 0}
          <!-- Empty State -->
          <div class="bg-surface border border-hairline rounded-lg">
            <div class="flex flex-col items-center justify-center py-16 px-6">
              <div class="w-12 h-12 rounded-full bg-surface-elevated flex items-center justify-center mb-4">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-stone">
                  <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </div>
              <p class="text-mute text-sm mb-1">暂无抓取记录</p>
              <p class="text-stone text-xs">完成首次抓取后，数据将显示在此处</p>
            </div>
          </div>

        {:else}
          <!-- Task Card List -->
          {#each taskHistory as task (task.id)}
            <div class="task-card-wrapper" role="listitem">
              <!-- CustomEvent delegation for taskclick and openfolder -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                use:delegateEvents={{
                  taskclick: () => handleTaskClick(task.id),
                  openfolder: (e: Event) => handleOpenFolder((e as CustomEvent).detail),
                }}
              >
                <TaskCard {task} onDelete={handleDeleteTask} />
              </div>

              <!-- Inline Detail Panel (expanded below card) -->
              {#if selectedTaskId === task.id && taskDetail}
                {@const product = taskDetail.product}
                <div class="mt-2 bg-surface-elevated border border-hairline rounded-lg p-4 space-y-3">
                  <h3 class="text-sm font-medium text-on-dark">任务详情</h3>

                  {#if product}
                    <!-- Product Info Grid -->
                    <div class="grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <dt class="text-mute text-xs mb-0.5">商品标题</dt>
                        <dd class="text-body">{product.title}</dd>
                      </div>
                      <div>
                        <dt class="text-mute text-xs mb-0.5">价格</dt>
                        <dd class="text-body font-mono text-xs">{formatPrice(product.price)}</dd>
                      </div>
                      <div>
                        <dt class="text-mute text-xs mb-0.5">店铺</dt>
                        <dd class="text-body">{product.shop.name}</dd>
                      </div>
                      <div>
                        <dt class="text-mute text-xs mb-0.5">SKU 数量</dt>
                        <dd class="text-body">{product.skus.length}</dd>
                      </div>
                      <div>
                        <dt class="text-mute text-xs mb-0.5">主图数量</dt>
                        <dd class="text-body">{product.gallery.length}</dd>
                      </div>
                      <div>
                        <dt class="text-mute text-xs mb-0.5">详情图片数</dt>
                        <dd class="text-body">{product.detail_images.length}</dd>
                      </div>
                    </div>

                    <!-- Cover Image Preview -->
                    {#if product.cover.local_path || product.cover.thumbnail_url}
                      <div>
                        <dt class="text-mute text-xs mb-1">封面预览</dt>
                        <img
                          src={product.cover.local_path ?? product.cover.thumbnail_url}
                          alt={product.title}
                          class="w-full max-w-[200px] h-auto rounded-md border border-hairline object-cover"
                        />
                      </div>
                    {/if}
                  {:else}
                    <p class="text-mute text-sm">暂无商品数据（解析可能失败）</p>
                  {/if}

                  <!-- Image Records Summary -->
                  {#if taskDetail.images.length > 0}
                    <div>
                      <dt class="text-mute text-xs mb-1">图片资源 ({taskDetail.images.length})</dt>
                      <dd class="text-body text-xs">
                        成功: {taskDetail.images.filter((img) => img.local_path).length} /
                        总计: {taskDetail.images.length}
                      </dd>
                    </div>
                  {/if}

                  <!-- Meta Info -->
                  <div class="pt-2 border-t border-hairline flex items-center justify-between">
                    <span class="text-stone text-xs">抓取时间: {formatTime(task.created_at)}</span>
                    <div class="flex items-center gap-3">
                      {#if task.folder_path}
                        <button
                          type="button"
                          onclick={() => handleOpenFolder(task.folder_path ?? undefined)}
                          class="text-mute hover:text-on-dark transition-colors cursor-pointer bg-transparent border-none text-xs p-0 underline"
                        >
                          打开文件夹
                        </button>
                        <button
                          type="button"
                          onclick={() => handleResizeImages(task.id)}
                          class="text-mute hover:text-on-dark transition-colors cursor-pointer bg-transparent border-none text-xs p-0 underline"
                        >
                          压缩图片
                        </button>
                      {/if}
                    </div>
                  </div>

                  <!-- Resize Result Feedback -->
                  {#if resizeLoading}
                    <p class="text-mute text-xs pt-2">压缩中...</p>
                  {/if}
                  {#if resizeResult}
                    <p class="text-mute text-xs pt-2">
                      压缩完成：共 {resizeResult.total} 张，
                      已压缩 {resizeResult.resized} 张，
                      跳过 {resizeResult.skipped} 张
                      {#if resizeResult.failed > 0}
                        ，失败 {resizeResult.failed} 张
                      {/if}
                    </p>
                  {/if}
                  {#if resizeError}
                    <p class="text-accent-red text-xs pt-2">{resizeError}</p>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        {/if}

        <!-- Global Error Message -->
        {#if taskError && !loading}
          <p class="text-accent-red text-sm text-center">{taskError}</p>
        {/if}
      </section>
    </div>
  </main>
</div>


