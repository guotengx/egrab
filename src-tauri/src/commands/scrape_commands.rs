// EGrab - IPC Commands: Scrape
// start_scrape / cancel_scrape Tauri command implementations.
//
// Protocol reference: src/protocols/ipc-commands.ts

use crate::models::{ErrorCode, IpcError, ScrapeStep};
use crate::scraper::ScraperEngine;
use futures::FutureExt;
use tauri::Emitter;

/// Starts a single-product scrape task.
///
/// Corresponds to protocol: StartScrapeCommand
/// - name: `start_scrape`
/// - params: `{ url: string, force?: boolean }`
/// - returns: `TaskId` (string)
/// - errors: `URL_INVALID` | `UNSUPPORTED_PLATFORM` | `DUPLICATE_TASK` | `TASK_ALREADY_RUNNING`
///
/// The task_id is returned immediately. The actual scrape pipeline
/// (CDP navigation, parsing, downloading, saving) runs in a background
/// tokio task, with progress/results emitted as Tauri events.
#[tauri::command]
pub async fn start_scrape(
    app_handle: tauri::AppHandle,
    url: String,
    force: Option<bool>,
) -> Result<String, IpcError> {
    // Validate URL is not empty.
    if url.trim().is_empty() {
        return Err(IpcError {
            code: ErrorCode::UrlInvalid,
            message: "URL must not be empty".to_string(),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        });
    }

    // Phase 1: Prepare the task (validate URL, create storage record).
    // This returns immediately with a task_id.
    let engine = ScraperEngine::new(app_handle.clone());
    let task_id = engine.prepare_task(&url, force).await?;

    // Phase 2: Run the scrape pipeline in a background tokio task.
    // The IPC command returns immediately; the frontend receives
    // progress via scrape:progress events.
    let app_handle_bg = app_handle.clone();
    let url_bg = url.clone();
    let tid = task_id.clone();
    tracing::info!(task_id = %tid, "Spawning background scrape task");
    tokio::spawn(async move {
        tracing::info!(task_id = %tid, "Background scrape task started");
        let engine = ScraperEngine::new(app_handle_bg.clone());

        // Wrap run_scrape in catch_unwind to prevent silent panics.
        // tokio::spawn swallows panics in the spawned future by default,
        // which would leave the frontend stuck at 0% with no error feedback.
        let result = std::panic::AssertUnwindSafe(async {
            engine.run_scrape(tid.clone(), url_bg).await;
        });
        if let Err(panic_err) = futures::FutureExt::catch_unwind(result).await {
            let panic_msg = if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else {
                format!("Unknown panic: {:?}", panic_err)
            };
            tracing::error!(
                task_id = %tid,
                panic = %panic_msg,
                "run_scrape panicked inside tokio::spawn"
            );
            // Emit error event so the frontend doesn't remain stuck at 0%.
            let payload = serde_json::json!({
                "task_id": tid,
                "error": format!("Scrape task panicked: {}", panic_msg),
                "recoverable": false,
            });
            let _ = app_handle_bg.emit("scrape:error", &payload);
        }
    });

    Ok(task_id)
}

/// Cancels a running scrape task.
///
/// Corresponds to protocol: CancelScrapeCommand
/// - name: `cancel_scrape`
/// - params: `{ task_id: string }`
/// - returns: `true` if the task was successfully cancelled
/// - errors: `TASK_NOT_FOUND` | `TASK_CANCELLED`
#[tauri::command]
pub async fn cancel_scrape(
    app_handle: tauri::AppHandle,
    task_id: String,
) -> Result<bool, IpcError> {
    let engine = ScraperEngine::new(app_handle);
    engine.cancel_scrape(&task_id).await
}
