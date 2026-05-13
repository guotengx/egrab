<!-- EGrab - URL Input Component -->
<!-- URL input with platform auto-detection; delegates scrape trigger to parent via onSubmit callback -->
<!-- Design: Raycast text-input + badge-info-soft platform tag + button-primary -->

<script lang="ts">
  import type { KnownPlatform } from '../protocols';

  interface Props {
    onSubmit: (url: string, platform: string, force: boolean) => void;
    loading?: boolean;
  }

  let { onSubmit, loading = false }: Props = $props();

  // --- State ---
  let urlInput = $state<string>('');
  let force = $state<boolean>(false);

  // --- Platform detection patterns (from docs/protocols/ipc-commands.md) ---
  const PLATFORM_PATTERNS: { pattern: RegExp; platform: KnownPlatform; label: string }[] = [
    { pattern: /^https?:\/\/item\.taobao\.com\/item\.htm\?.*\bid=\d+/i, platform: 'taobao', label: '淘宝' },
    { pattern: /^https?:\/\/detail\.tmall\.com\/item\.htm\?.*\bid=\d+/i, platform: 'tmall', label: '天猫' },
    { pattern: /^https?:\/\/item\.jd\.com\/\d+\.html/i, platform: 'jd', label: '京东' },
  ];

  /** Normalize URL: ensure https prefix. */
  function normalizeUrl(raw: string): string {
    const trimmed = raw.trim();
    if (/^http:\/\//i.test(trimmed)) {
      return trimmed.replace(/^http:\/\//i, 'https://');
    }
    if (!/^https:\/\//i.test(trimmed)) {
      return `https://${trimmed}`;
    }
    return trimmed;
  }

  /** Detect platform from URL. Returns null if not recognized. */
  function detectPlatform(url: string): { platform: KnownPlatform; label: string } | null {
    if (!url || url.length === 0) return null;
    const normalized = normalizeUrl(url);
    for (const entry of PLATFORM_PATTERNS) {
      if (entry.pattern.test(normalized)) {
        return { platform: entry.platform, label: entry.label };
      }
    }
    return null;
  }

  // --- Derived values ---
  let normalizedUrl = $derived(normalizeUrl(urlInput));
  let detectedPlatform = $derived(detectPlatform(urlInput));
  let isUrlValid = $derived(urlInput.trim().length > 0 && detectedPlatform !== null);

  let errorMessage = $derived.by(() => {
    if (urlInput.trim().length === 0) return '';
    if (detectedPlatform === null) return '不支持的平台';
    return '';
  });

  /** Handle form submission: validate URL and delegate to parent via onSubmit callback. */
  function handleSubmit(): void {
    if (!isUrlValid || loading) return;

    onSubmit(normalizedUrl, detectedPlatform!.platform, force);
    // Clear input after submission
    urlInput = '';
    force = false;
  }
</script>

<div class="space-y-3">
  <!-- Input Row: URL field + Submit button -->
  <div class="flex items-center gap-3">
    <!-- URL Text Input -->
    <div class="flex-1 relative">
      <input
        type="text"
        bind:value={urlInput}
        placeholder="输入商品链接..."
        disabled={loading}
        class="w-full h-9 px-3 py-2 bg-surface-elevated border border-hairline rounded-md text-on-dark text-sm placeholder:text-ash focus:border-hairline-strong focus:outline-none transition-colors disabled:opacity-50"
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            handleSubmit();
          }
        }}
      />
    </div>

    <!-- Start Scrape Button -->
    <button
      onclick={handleSubmit}
      disabled={!isUrlValid || loading}
      class="bg-primary text-on-primary rounded-md font-medium px-6 py-2 text-sm transition-colors cursor-pointer shrink-0 disabled:bg-surface-elevated disabled:text-ash disabled:cursor-not-allowed hover:bg-primary-pressed"
    >
      {loading ? '抓取中...' : '开始抓取'}
    </button>
  </div>

  <!-- Force Re-scrape Toggle -->
  <label class="force-toggle flex items-center gap-1.5 cursor-pointer select-none">
    <input
      type="checkbox"
      bind:checked={force}
      disabled={loading}
      class="w-3.5 h-3.5 rounded cursor-pointer accent-white disabled:cursor-not-allowed disabled:opacity-50"
    />
    <span class="text-body text-xs leading-none">强制重新抓取</span>
  </label>

  <!-- Platform Tag / Error Message Area -->
  <div class="min-h-[20px] flex items-center gap-2">
    {#if errorMessage && !detectedPlatform}
      <!-- Unsupported platform error -->
      <span class="text-accent-red text-xs">{errorMessage}</span>
    {:else if detectedPlatform}
      <!-- Platform badge -->
      <span class="bg-accent-blue-soft text-accent-blue rounded-xs px-2 py-0.5 text-xs">
        {detectedPlatform.label}
      </span>
    {/if}
  </div>
</div>
