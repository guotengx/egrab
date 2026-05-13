<!-- EGrab - Status Bar Component -->
<!-- CDP connection status indicator with click-to-connect/disconnect -->
<!-- Design: compact dot + label, interactive, Raycast dark theme -->
<!-- Delegates all CDP operations to connectionStore for centralized state management -->

<script lang="ts">
import { connectionStore } from '../stores/connection.svelte';
import { configStore } from '../stores/config.svelte';

  // --- Derived from stores ---
  let connectionState = $derived(connectionStore.state);
  let isOperating = $derived(connectionStore.isOperating);

  /** Map connection state type to accent color class for status dot. */
  let statusColor = $derived.by(() => {
    switch (connectionState.type) {
      case 'Connected':
        return 'bg-accent-green';
      case 'Connecting':
      case 'Reconnecting':
        return 'bg-accent-yellow';
      default:
        return 'bg-accent-red';
    }
  });

  /** Whether the status dot should pulse (connecting/reconnecting states). */
  let shouldPulse = $derived(
    connectionState.type === 'Connecting' || connectionState.type === 'Reconnecting'
  );

  /** Human-readable status text based on connection state. */
  let statusText = $derived.by(() => {
    switch (connectionState.type) {
      case 'Connected':
        return `${connectionState.browser_version}`;
      case 'Connecting':
        return '连接中...';
      case 'Reconnecting':
        return `重连中 (${connectionState.attempt})`;
      case 'Failed':
        return `失败: ${connectionState.reason}`;
      default:
        return '未连接';
    }
  });

  /** Whether clicking should trigger connect action. */
  let canConnect = $derived(
    connectionState.type === 'Disconnected' || connectionState.type === 'Failed'
  );

  /** Whether clicking should trigger disconnect action. */
  let canDisconnect = $derived(connectionState.type === 'Connected');

  /** Whether the component is in an interactive (clickable) state. */
  let isInteractive = $derived((canConnect || canDisconnect) && !isOperating);

  /** Handle click: connect or disconnect via connectionStore. */
  async function handleClick(): Promise<void> {
    if (!isInteractive || isOperating) return;

    if (canConnect) {
      await connectionStore.connect(configStore.config.cdp_port);
    } else if (canDisconnect) {
      await connectionStore.disconnect();
    }
  }
</script>

<button
  type="button"
  disabled={!isInteractive}
  class="flex items-center gap-2 bg-transparent border-none p-0 {isInteractive ? 'cursor-pointer' : ''}"
  title={isInteractive ? (canConnect ? '点击连接 CDP' : '点击断开连接') : statusText}
  onclick={handleClick}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  }}
>
  <!-- Status dot: 8x8px rounded-full with semantic color + optional pulse -->
  <span
    class="inline-block w-2 h-2 rounded-full {statusColor} {shouldPulse ? 'animate-status-pulse' : ''}"
  ></span>

  <!-- Status label: muted caption text -->
  <span class="text-xs text-mute">
    {#if isOperating}
      处理中...
    {:else}
      {statusText}
    {/if}
  </span>
</button>
