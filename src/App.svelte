<!-- EGrab - Root Application Component -->
<!-- Top-level layout with primary navigation bar and page routing -->
<!-- Design: Raycast-inspired dark theme, primary-nav + pill-tab navigation -->

<script lang="ts">
  import Home from './pages/Home.svelte';
  import Progress from './pages/Progress.svelte';
  import Archive from './pages/Archive.svelte';
  import Settings from './pages/Settings.svelte';
  import StatusBar from './components/StatusBar.svelte';

  type Page = 'home' | 'progress' | 'archive' | 'settings';

  let currentPage = $state<Page>('home');

  /** Navigation callback for child components to request page changes. */
  function navigateTo(page: Page): void {
    currentPage = page;
  }
</script>

<div class="h-screen flex flex-col bg-canvas text-body" style="font-feature-settings: 'calt', 'kern', 'liga', 'ss03';">
  <!-- Primary Nav: DESIGN.md `primary-nav` spec -->
  <!-- bg-canvas, h-14 (~56px), hairline bottom border, body-sm-strong font -->
  <nav class="bg-canvas border-b border-hairline h-14 px-6 flex items-center justify-between shrink-0">
    <!-- Left: Brand (Raycast-style wordmark) -->
    <div class="flex items-center gap-2">
      <span class="text-on-dark text-[13px] font-medium tracking-wide brand-letter">EGrab</span>
    </div>

    <!-- Center: Nav Tabs (DESIGN.md `pill-tab` / `pill-tab-active` pattern) -->
    <!-- Default: transparent bg, text-body, body-sm, px-2.5 py-1, rounded-full -->
    <!-- Active: bg-surface-elevated, text-on-dark -->
    <div class="flex items-center gap-1">
      <button
        class="px-2.5 py-1 text-[13px] rounded-full transition-colors cursor-pointer {currentPage === 'home'
          ? 'bg-surface-elevated text-on-dark font-medium'
          : 'text-body hover:text-on-dark'}"
        onclick={() => (currentPage = 'home')}
      >
        首页
      </button>
      <button
        class="px-2.5 py-1 text-[13px] rounded-full transition-colors cursor-pointer {currentPage === 'archive'
          ? 'bg-surface-elevated text-on-dark font-medium'
          : 'text-body hover:text-on-dark'}"
        onclick={() => (currentPage = 'archive')}
      >
        存档
      </button>
      <button
        class="px-2.5 py-1 text-[13px] rounded-full transition-colors cursor-pointer {currentPage === 'settings'
          ? 'bg-surface-elevated text-on-dark font-medium'
          : 'text-body hover:text-on-dark'}"
        onclick={() => (currentPage = 'settings')}
      >
        设置
      </button>
    </div>

    <!-- Right: CDP Status Indicator (embedded StatusBar) -->
    <StatusBar />
  </nav>

  <!-- Page Content Area -->
  <div class="flex-1 overflow-auto">
    {#if currentPage === 'home'}
      <Home onNavigate={(page) => navigateTo(page as Page)} />
    {:else if currentPage === 'progress'}
      <Progress onNavigate={(page) => navigateTo(page as Page)} />
    {:else if currentPage === 'archive'}
      <Archive onNavigate={(page) => navigateTo(page as Page)} />
    {:else if currentPage === 'settings'}
      <Settings />
    {/if}
  </div>
</div>
