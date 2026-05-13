<!-- EGrab - Task Card Component -->
<!-- Displays a task summary as a horizontal card with cover thumbnail + info -->
<!-- Design: Raycast dark theme, store-extension-card pattern -->

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { TaskSummary, TaskStatus } from '../protocols';

  interface Props {
    task: TaskSummary;
    onDelete?: (taskId: string) => void;
  }

  let { task, onDelete }: Props = $props();

  let coverUrl = $state<string | null>(null);

  // Load cover image via IPC when task changes
  $effect(() => {
    coverUrl = null;
    if (task.id) {
      invoke<string>('get_cover_image', { taskId: task.id })
        .then(url => { coverUrl = url; })
        .catch(() => { coverUrl = null; });
    }
  });

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    if (onDelete) onDelete(task.id);
  }

  /** Platform label mapping. */
  function getPlatformLabel(platform: string): string {
    const labels: Record<string, string> = {
      taobao: '淘宝',
      tmall: '天猫',
      jd: '京东',
    };
    return labels[platform] ?? platform;
  }

  /** Status badge styling based on task status. */
  function getStatusStyle(status: TaskStatus): { bg: string; text: string; label: string } {
    const styles: Record<TaskStatus, { bg: string; text: string; label: string }> = {
      success: { bg: 'bg-accent-green-soft', text: 'text-accent-green', label: '成功' },
      failed: { bg: 'bg-accent-red-soft', text: 'text-accent-red', label: '失败' },
      partial: { bg: 'bg-accent-yellow-soft', text: 'text-accent-yellow', label: '部分完成' },
      running: { bg: 'bg-accent-blue-soft', text: 'text-accent-blue', label: '运行中' },
      pending: { bg: 'bg-surface-elevated', text: 'text-ash', label: '等待中' },
      cancelled: { bg: 'bg-surface-elevated', text: 'text-ash', label: '已取消' },
    };
    return styles[status] ?? styles.pending;
  }

  /** Format ISO timestamp to locale-friendly short string. */
  function formatTime(isoString: string): string {
    try {
      const date = new Date(isoString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffMin = Math.floor(diffMs / 60000);
      const diffHour = Math.floor(diffMs / 3600000);
      const diffDay = Math.floor(diffMs / 86400000);

      if (diffMin < 1) return '刚刚';
      if (diffMin < 60) return `${diffMin} 分钟前`;
      if (diffHour < 24) return `${diffHour} 小时前`;
      if (diffDay < 7) return `${diffDay} 天前`;

      return date.toLocaleDateString('zh-CN', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return isoString;
    }
  }

  let statusStyle = $derived(getStatusStyle(task.status));
</script>

<!-- Card Container: surface bg, hairline border, hover effect -->
<div
  class="bg-surface border border-hairline rounded-lg p-4 hover:border-hairline-strong transition-colors cursor-pointer"
  role="button"
  tabindex="0"
  onclick={() => {
    // Click event handled by parent via dispatch or callback
    const event = new CustomEvent('taskclick', { detail: task.id, bubbles: true });
    (event.target as HTMLElement).closest('.task-card')?.dispatchEvent(event);
  }}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      const event = new CustomEvent('taskclick', { detail: task.id, bubbles: true });
      (e.currentTarget as HTMLElement).dispatchEvent(event);
    }
  }}
>
  <div class="task-card flex items-center gap-4">
    <!-- Cover Thumbnail: 64x64, rounded, placeholder when no image -->
    <div class="w-16 h-16 rounded-md bg-surface-elevated shrink-0 flex items-center justify-center overflow-hidden">
      {#if coverUrl}
        <img
          src={coverUrl}
          alt={task.title}
          class="w-full h-full object-cover"
        />
      {:else}
        <!-- Placeholder icon -->
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-stone">
          <rect x="3" y="3" width="18" height="18" rx="2" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="8.5" cy="8.5" r="1.5" fill="currentColor"/>
          <path d="M21 15l-5-5L5 21" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      {/if}
    </div>

    <!-- Info Column: title, platform tag, time -->
    <div class="flex-1 min-w-0 space-y-1">
      <!-- Title row: title + status badge -->
      <div class="flex items-center gap-2">
        <span class="text-on-dark text-sm font-medium truncate">{task.title}</span>
        <span class="{statusStyle.bg} {statusStyle.text} rounded-xs px-2 py-0.5 text-xs shrink-0">
          {statusStyle.label}
        </span>
      </div>

      <!-- Meta row: platform tag + time -->
      <div class="flex items-center gap-2">
        <span class="bg-accent-blue-soft text-accent-blue rounded-xs px-2 py-0.5 text-xs">
          {getPlatformLabel(task.platform)}
        </span>
        <span class="text-mute text-xs">{formatTime(task.created_at)}</span>
      </div>
    </div>

    <!-- Delete Button -->
    <button
      type="button"
      class="text-mute hover:text-accent-red transition-colors shrink-0 p-1 rounded cursor-pointer"
      title="删除"
      onclick={handleDelete}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M2.5 4h11M5.5 4v-1a1 1 0 011-1h3a1 1 0 011 1v1M6.5 7v5M9.5 7v5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>

    <!-- Open Folder Button -->
    <button
      type="button"
      class="text-mute hover:text-on-dark transition-colors shrink-0 p-1 rounded cursor-pointer"
      title="打开文件夹"
      onclick={(e) => {
        e.stopPropagation();
        const event = new CustomEvent('openfolder', { detail: task.folder_path, bubbles: true });
        (e.currentTarget as HTMLElement).dispatchEvent(event);
      }}
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M1.5 3.5h4.586a1 1 0 01.707.293l.707 1.414A1 1 0 009.207 5.5H14.5a1 1 0 011 1v6a1 1 0 01-1 1h-13a1 1 0 01-1-1v-8a1 1 0 011-1z" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
  </div>
</div>
