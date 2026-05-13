<!-- EGrab - Progress Page -->
<!-- Real-time scrape progress display with ProgressBar, error list, and cancel -->
<!-- Design: Raycast dark theme, centered max-w-2xl layout -->

<script lang="ts">
  import ProgressBar from '../components/ProgressBar.svelte';
  import { tasksStore } from '../stores/tasks.svelte';

  interface Props {
    onNavigate?: (page: string) => void;
  }

  let { onNavigate }: Props = $props();

  // --- Derived from store ---
  let currentTask = $derived(tasksStore.currentTask);
  let taskError = $derived(tasksStore.error);

  /** Navigate back to home page. */
  function handleBack(): void {
    if (onNavigate) {
      onNavigate('home');
    }
  }

  /** Navigate to archive page after completion. */
  function handleViewArchive(): void {
    if (onNavigate) {
      onNavigate('archive');
    }
  }

  /** Handle cancel button click. */
  async function handleCancel(): Promise<void> {
    await tasksStore.cancelScrape();
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <main class="flex-1 p-6 overflow-auto">
    <div class="max-w-2xl mx-auto space-y-6">
      <!-- Header: Back Button + Title -->
      <div class="flex items-center gap-3 pt-4 pb-2">
        <!-- Back Button -->
        <button
          type="button"
          onclick={handleBack}
          class="flex items-center gap-1 text-mute hover:text-on-dark transition-colors cursor-pointer bg-transparent border-none p-0 text-sm"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M10 12L6 8l4-4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          返回
        </button>
        <h1 class="text-xl font-medium text-ink tracking-tight">抓取进度</h1>
      </div>

      {#if !currentTask}
        <!-- No Active Task State -->
        <section class="bg-surface border border-hairline rounded-lg p-10">
          <div class="flex flex-col items-center justify-center text-center">
            <div class="w-12 h-12 rounded-full bg-surface-elevated flex items-center justify-center mb-4">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-stone">
                <circle cx="12" cy="12" r="9"/>
                <path d="M12 7v5l3 3" stroke-linecap="round"/>
              </svg>
            </div>
            <p class="text-body text-sm mb-1">没有正在进行的任务</p>
            <p class="text-mute text-xs mb-4">在首页输入商品链接开始抓取</p>
            {#if onNavigate}
              <button
                type="button"
                onclick={handleBack}
                class="bg-primary text-on-primary rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer hover:bg-primary-pressed"
              >
                返回首页
              </button>
            {/if}
          </div>
        </section>

      {:else}
        <!-- Active Task: Progress Display -->
        <ProgressBar
          percent={currentTask.percent}
          step={currentTask.step}
          message={currentTask.message}
        />

        <!-- Error List (if any) -->
        {#if currentTask.errors.length > 0}
          <section class="bg-surface border border-hairline rounded-lg p-5 space-y-3">
            <h2 class="text-sm font-medium text-accent-red">错误 / 警告 ({currentTask.errors.length})</h2>
            <div class="space-y-2">
              {#each currentTask.errors as err, i (i)}
                <div class="bg-surface-elevated rounded-md p-3 space-y-1">
                  <div class="flex items-center gap-2">
                    <span class="text-accent-red text-xs font-mono">{err.code}</span>
                    <span class="text-stone text-xs">{err.step}</span>
                  </div>
                  <p class="text-body text-xs">{err.message}</p>
                  {#if err.recoverable}
                    <span class="bg-accent-yellow-soft text-accent-yellow rounded-xs px-2 py-0.5 text-xs">可恢复</span>
                  {/if}
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <!-- Action Buttons -->
        <div class="flex items-center gap-3">
          {#if currentTask.step !== 'completed' && currentTask.step !== 'failed'}
            <!-- Cancel Button (only show when task is still running) -->
            <button
              type="button"
              onclick={handleCancel}
              class="bg-surface-elevated text-on-dark border border-hairline rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer hover:border-hairline-strong"
            >
              取消抓取
            </button>
          {:else if currentTask.step === 'completed'}
            <!-- Completed: View in Archive -->
            <button
              type="button"
              onclick={handleViewArchive}
              class="bg-primary text-on-primary rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer hover:bg-primary-pressed"
            >
              查看存档
            </button>
          {:else}
            <!-- Failed: Retry from Home -->
            <button
              type="button"
              onclick={handleBack}
              class="bg-primary text-on-primary rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer hover:bg-primary-pressed"
            >
              返回重试
            </button>
          {/if}

          {#if taskError}
            <span class="text-accent-red text-xs">{taskError}</span>
          {/if}
        </div>
      {/if}
    </div>
  </main>
</div>
