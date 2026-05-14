// EGrab - IPC Commands: Image Resize
// Provides Tauri commands for resizing oversized product images.

use crate::models::IpcError;
use crate::resize::resize_images_in_folder;
use crate::storage::StorageEngine;

#[tauri::command]
pub async fn resize_images(
    task_id: String,
    storage: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
) -> Result<crate::resize::ResizeResult, IpcError> {
    let folder_path = {
        let guard = storage.lock().await;
        let detail = guard.get_task_detail(&task_id)?;
        detail.task.folder_path.clone().ok_or_else(|| IpcError {
            code: crate::models::ErrorCode::TaskNotFound,
            message: "Task has no folder_path".to_string(),
            recoverable: false,
            step: Some(crate::models::ScrapeStep::Saving),
            details: None,
        })?
    };

    // Run the CPU-bound resize in a blocking task to avoid blocking the async runtime.
    let result = tokio::task::spawn_blocking(move || resize_images_in_folder(&folder_path))
        .await
        .map_err(|e| IpcError {
            code: crate::models::ErrorCode::UnknownError,
            message: format!("Resize task panicked: {}", e),
            recoverable: false,
            step: Some(crate::models::ScrapeStep::Saving),
            details: None,
        })??;

    Ok(result)
}
