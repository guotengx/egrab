<!-- EGrab - Progress Bar Component -->
<!-- Displays scrape progress with percentage, current step, and step list -->
<!-- Design: Raycast dark theme, surface card with hairline border -->

<script lang="ts">
  import type { ScrapeStep } from '../protocols';

  interface Props {
    percent: number;
    step: ScrapeStep;
    message: string;
  }

  let { percent, step, message }: Props = $props();

  /** Ordered scrape steps for display in the step list. */
  const SCRAPE_STEPS: { key: ScrapeStep; label: string }[] = [
    { key: 'connecting', label: '连接浏览器' },
    { key: 'page_loading', label: '页面加载' },
    { key: 'parsing', label: '数据解析' },
    { key: 'downloading', label: '图片下载' },
    { key: 'saving', label: '存档写入' },
  ];

  /** Get the index of the current step in the ordered list. */
  function getCurrentStepIndex(): number {
    return SCRAPE_STEPS.findIndex((s) => s.key === step);
  }

  /** Determine the display state of a step relative to the current step. */
  function getStepState(stepKey: ScrapeStep): 'completed' | 'current' | 'pending' {
    const currentIndex = getCurrentStepIndex();
    const stepIndex = SCRAPE_STEPS.findIndex((s) => s.key === stepKey);

    if (stepIndex < 0) return 'pending';
    if (stepIndex < currentIndex) return 'completed';
    if (stepIndex === currentIndex) return 'current';
    return 'pending';
  }

  /** Clamped percent value for display (0-100). */
  let clampedPercent = $derived(Math.max(0, Math.min(100, percent)));

  /** Whether the task has completed or failed. */
  let isTerminal = $derived(step === 'completed' || step === 'failed');
</script>

<div class="bg-surface border border-hairline rounded-lg p-5 space-y-4">
  <!-- Progress Bar Track + Fill -->
  <div class="space-y-2">
    <div class="bg-surface-elevated h-1.5 rounded-full overflow-hidden">
      <div
        class="h-full bg-on-dark rounded-full transition-all duration-300 ease-out"
        style="width: {clampedPercent}%"
      ></div>
    </div>

    <!-- Percent + Message Row -->
    <div class="flex items-center justify-between">
      <span class="text-body text-sm">{message}</span>
      <span class="text-mute text-xs font-mono">{Math.round(clampedPercent)}%</span>
    </div>
  </div>

  <!-- Step List -->
  <div class="space-y-1.5 pt-1">
    {#each SCRAPE_STEPS as scrapeStep}
      {@const stepState = getStepState(scrapeStep.key)}
      <div class="flex items-center gap-2.5 text-sm">
        <!-- Step Indicator Icon -->
        {#if stepState === 'completed'}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="shrink-0">
            <path d="M11.5 4L5.25 10.25L2.5 7.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-accent-green"/>
          </svg>
        {:else if stepState === 'current'}
          <span class="text-on-dark font-medium shrink-0">→</span>
        {:else}
          <span class="text-stone shrink-0">○</span>
        {/if}

        <!-- Step Label -->
        <span
          class:text-accent-green={stepState === 'completed'}
          class:text-on-dark={stepState === 'current'}
          class:font-medium={stepState === 'current'}
          class:text-stone={stepState === 'pending'}
        >
          {scrapeStep.label}
        </span>
      </div>
    {/each}

    <!-- Terminal State Step -->
    {#if isTerminal}
      <div class="flex items-center gap-2.5 text-sm">
        {#if step === 'completed'}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="shrink-0">
            <path d="M11.5 4L5.25 10.25L2.5 7.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-accent-green"/>
          </svg>
          <span class="text-accent-green font-medium">完成</span>
        {:else}
          <span class="text-accent-red shrink-0">✕</span>
          <span class="text-accent-red font-medium">失败</span>
        {/if}
      </div>
    {/if}
  </div>
</div>
