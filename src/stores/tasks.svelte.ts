// EGrab - Tasks Store
// Manages scrape task state: current task progress, task history, and task detail
// Listens to backend events (scrape:progress, scrape:complete, scrape:error)
// Delegates IPC calls to service layer

import type {
  ScrapeStep,
  TaskFilter,
  TaskSummary,
  TaskDetail,
  ScrapeErrorInfo,
} from '../protocols';
import {
  startScrape as ipcStartScrape,
  cancelScrape as ipcCancelScrape,
  getTaskHistory as ipcGetTaskHistory,
  getTaskDetail as ipcGetTaskDetail,
  deleteTask as ipcDeleteTask,
  openFolder as ipcOpenFolder,
} from '../services/ipc';
import {
  onScrapeProgress,
  onScrapeComplete,
  onScrapeError,
} from '../services/events';

/** Extract human-readable error message from Tauri IpcError or standard Error. */
function extractErrorMessage(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    return String((err as { message: unknown }).message);
  }
  if (err instanceof Error) return err.message;
  return String(err);
}

/** Current running task state for Progress page */
export interface CurrentTask {
  task_id: string;
  url: string;
  platform: string;
  percent: number;
  step: ScrapeStep;
  message: string;
  errors: ScrapeErrorInfo[];
}

/**
 * Reactive tasks store using Svelte 5 runes.
 * Provides current task tracking, history loading, and event-driven updates.
 */
function createTasksStore() {
  // --- Core State ---
  let currentTask = $state<CurrentTask | null>(null);
  let taskHistory = $state<TaskSummary[]>([]);
  let taskDetail = $state<TaskDetail | null>(null);
  let loading = $state<boolean>(false);
  let error = $state<string | null>(null);

  // --- Event Listener Cleanup ---
  let unlistenProgress: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  // --- Event Listener State ---
  let listenersReady = $state<boolean>(false);
  let progressTimeoutId: ReturnType<typeof setTimeout> | null = null;

  /** Maximum retry attempts for event listener registration */
  const MAX_LISTENER_RETRIES = 3;
  /** Delay between retry attempts in ms */
  const LISTENER_RETRY_DELAY_MS = 500;
  /** Timeout in ms after startScrape to detect missing progress events */
  const PROGRESS_TIMEOUT_MS = 5_000;

  // --- Register Event Listeners with retry mechanism ---
  async function setupEventListeners(): Promise<void> {
    let retries = MAX_LISTENER_RETRIES;
    while (retries > 0) {
      try {
        console.log(`[tasks] Registering event listeners (attempt ${MAX_LISTENER_RETRIES - retries + 1}/${MAX_LISTENER_RETRIES})...`);

        unlistenProgress = await onScrapeProgress((payload) => {
          console.log('[tasks] Received scrape:progress', payload);
          // Clear progress timeout on first progress event received
          if (progressTimeoutId !== null) {
            clearTimeout(progressTimeoutId);
            progressTimeoutId = null;
          }
          if (currentTask && currentTask.task_id === payload.task_id) {
            currentTask = {
              ...currentTask,
              percent: payload.percent,
              step: payload.step,
              message: payload.message,
            };
          }
        });
        console.log('[tasks] scrape:progress listener registered');

        unlistenComplete = await onScrapeComplete((payload) => {
          console.log('[tasks] Received scrape:complete', payload);
          if (progressTimeoutId !== null) {
            clearTimeout(progressTimeoutId);
            progressTimeoutId = null;
          }
          if (currentTask && currentTask.task_id === payload.task_id) {
            currentTask = {
              ...currentTask,
              percent: 100,
              step: 'completed',
              message: '抓取完成',
            };
            // Auto-refresh history after completion
            loadHistory();
          }
        });
        console.log('[tasks] scrape:complete listener registered');

        unlistenError = await onScrapeError((payload) => {
          console.log('[tasks] Received scrape:error', payload);
          if (progressTimeoutId !== null) {
            clearTimeout(progressTimeoutId);
            progressTimeoutId = null;
          }
          if (currentTask && currentTask.task_id === payload.task_id) {
            const newError: ScrapeErrorInfo = {
              step: payload.step,
              code: payload.error_code,
              message: payload.error,
              recoverable: payload.recoverable,
            };
            currentTask = {
              ...currentTask,
              errors: [...currentTask.errors, newError],
              step: 'failed',
            };
          }
        });
        console.log('[tasks] scrape:error listener registered');

        listenersReady = true;
        console.log('[tasks] All event listeners registered successfully');
        return;
      } catch (err) {
        retries--;
        const retryMsg = retries > 0
          ? `retrying (${retries} left)...`
          : 'giving up.';
        console.warn(
          `[tasks] Failed to register event listeners, ${retryMsg}`,
          err
        );
        if (retries > 0) {
          await new Promise((resolve) => setTimeout(resolve, LISTENER_RETRY_DELAY_MS));
        }
      }
    }
    console.error(`[tasks] Failed to register event listeners after ${MAX_LISTENER_RETRIES} retries`);
    listenersReady = false;
  }

  /**
   * Start a progress timeout watchdog.
   * If no progress/complete/error event arrives within PROGRESS_TIMEOUT_MS,
   * proactively query the backend for task status as a fallback.
   */
  function startProgressWatchdog(taskId: string): void {
    clearProgressWatchdog();
    progressTimeoutId = setTimeout(async () => {
      console.warn(`[tasks] No progress event received within ${PROGRESS_TIMEOUT_MS}ms, querying backend for task ${taskId}`);
      try {
        const detail = await ipcGetTaskDetail(taskId);
        if (currentTask && currentTask.task_id === taskId) {
          // Map task detail status back to UI state (TaskStatus: pending|running|success|failed|partial|cancelled)
          const taskStatus = detail.task?.status;
          if (taskStatus === 'success') {
            currentTask = {
              ...currentTask,
              percent: 100,
              step: 'completed',
              message: '抓取完成（通过状态查询确认）',
            };
            loadHistory();
          } else if (taskStatus === 'failed' || taskStatus === 'cancelled') {
            currentTask = {
              ...currentTask,
              step: 'failed',
              message: taskStatus === 'cancelled' ? '任务已取消（通过状态查询确认）' : '任务失败（通过状态查询确认）',
            };
          } else if (taskStatus === 'running' || taskStatus === 'pending') {
            // Task is still running but events aren't flowing; update message and keep waiting
            currentTask = {
              ...currentTask,
              message: '正在处理中（事件流可能中断，已自动查询状态）...',
            };
            // Restart watchdog for another round
            startProgressWatchdog(taskId);
          }
        }
      } catch (err) {
        console.error('[tasks] Progress watchdog fallback query failed:', err);
        if (currentTask && currentTask.task_id === taskId) {
          currentTask = {
            ...currentTask,
            message: '进度事件超时且状态查询失败，请检查连接',
          };
        }
      }
    }, PROGRESS_TIMEOUT_MS);
  }

  /** Clear the progress timeout watchdog. */
  function clearProgressWatchdog(): void {
    if (progressTimeoutId !== null) {
      clearTimeout(progressTimeoutId);
      progressTimeoutId = null;
    }
  }

  /** Clean up all event listeners and watchdog (call on app teardown). */
  function cleanup(): void {
    clearProgressWatchdog();
    unlistenProgress?.();
    unlistenComplete?.();
    unlistenError?.();
    unlistenProgress = null;
    unlistenComplete = null;
    unlistenError = null;
    listenersReady = false;
  }

  // Initialize listeners immediately; catch errors if Tauri WebView is not ready yet
  setupEventListeners().catch((err) => {
    console.error('[tasks] setupEventListeners() rejected:', err);
  });

  /** Start a scrape task. Sets currentTask and calls backend start_scrape. */
  async function startScrape(url: string, force: boolean = false): Promise<void> {
    loading = true;
    error = null;

    try {
      const taskId = await ipcStartScrape(url, force);

      // Determine platform from URL for display
      let platform = 'unknown';
      if (/taobao\.com|tmall\.com/i.test(url)) {
        platform = url.includes('tmall') ? 'tmall' : 'taobao';
      } else if (/jd\.com/i.test(url)) {
        platform = 'jd';
      }

      currentTask = {
        task_id: taskId,
        url,
        platform,
        percent: 0,
        step: 'connecting',
        message: '正在连接浏览器...',
        errors: [],
      };

      // Start progress timeout watchdog to detect missing events
      startProgressWatchdog(taskId);
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `启动抓取失败: ${message}`;
    } finally {
      loading = false;
    }
  }

  /** Cancel the currently running scrape task. */
  async function cancelScrape(): Promise<boolean> {
    if (!currentTask) return false;

    clearProgressWatchdog();

    try {
      const success = await ipcCancelScrape(currentTask.task_id);
      if (success) {
        currentTask = {
          ...currentTask,
          step: 'failed',
          message: '任务已取消',
        };
      }
      return success;
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `取消任务失败: ${message}`;
      return false;
    }
  }

  /** Load task history from backend with optional filter. */
  async function loadHistory(filter?: TaskFilter): Promise<void> {
    loading = true;
    error = null;

    try {
      const result = await ipcGetTaskHistory(filter ?? {});
      taskHistory = result;
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `加载历史记录失败: ${message}`;
    } finally {
      loading = false;
    }
  }

  /** Load detail for a specific task. */
  async function loadDetail(taskId: string): Promise<void> {
    loading = true;
    error = null;
    taskDetail = null;

    try {
      taskDetail = await ipcGetTaskDetail(taskId);
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `加载任务详情失败: ${message}`;
    } finally {
      loading = false;
    }
  }

  /** Delete a task and refresh the history list. */
  async function deleteTask(taskId: string): Promise<boolean> {
    loading = true;
    error = null;

    try {
      const success = await ipcDeleteTask(taskId);
      if (success) {
        // Remove from local history immediately
        taskHistory = taskHistory.filter((t) => t.id !== taskId);
        // Clear detail if the deleted task was selected
        if (taskDetail && taskDetail.task.id === taskId) {
          taskDetail = null;
        }
      }
      return success;
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `删除任务失败: ${message}`;
      return false;
    } finally {
      loading = false;
    }
  }

  /** Open a local folder in system file explorer. */
  async function openFolder(path: string): Promise<boolean> {
    try {
      return await ipcOpenFolder(path);
    } catch (err) {
      const message = extractErrorMessage(err);
      error = `打开文件夹失败: ${message}`;
      return false;
    }
  }

  /** Clear the current task state (e.g., after navigating away). */
  function clearCurrentTask(): void {
    clearProgressWatchdog();
    currentTask = null;
  }

  /** Clear the task detail view. */
  function clearTaskDetail(): void {
    taskDetail = null;
  }

  return {
    get currentTask(): CurrentTask | null {
      return currentTask;
    },
    get taskHistory(): TaskSummary[] {
      return taskHistory;
    },
    get taskDetail(): TaskDetail | null {
      return taskDetail;
    },
    get loading(): boolean {
      return loading;
    },
    get error(): string | null {
      return error;
    },
    get listenersReady(): boolean {
      return listenersReady;
    },
    startScrape,
    cancelScrape,
    loadHistory,
    loadDetail,
    deleteTask,
    openFolder,
    clearCurrentTask,
    clearTaskDetail,
    cleanup,
  };
}

export const tasksStore = createTasksStore();
