// EGrab - IPC Commands: Task
// get_task_history / get_task_detail / open_folder Tauri command implementations.
//
// Protocol reference: src/protocols/ipc-commands.ts

use crate::models::{ErrorCode, IpcError, TaskDetail, TaskFilter, TaskSummary};
use crate::storage::StorageEngine;

/// Queries task history with optional filters.
///
/// Corresponds to protocol: GetTaskHistoryCommand
/// - name: `get_task_history`
/// - params: `{ filter: TaskFilter }`
/// - returns: `TaskSummary[]`
#[tauri::command]
pub async fn get_task_history(
    state: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
    filter: TaskFilter,
) -> Result<Vec<TaskSummary>, IpcError> {
    let engine = state.lock().await;
    let tasks = engine.query_tasks(filter)?;
    // Debug: log cover info
    for t in &tasks {
        tracing::info!(
            "TaskSummary: id={}, folder={:?}, cover={:?}",
            t.id,
            t.folder_path,
            t.cover_path
        );
    }
    Ok(tasks)
}

/// Returns detailed information about a specific task.
///
/// Corresponds to protocol: GetTaskDetailCommand
/// - name: `get_task_detail`
/// - params: `{ task_id: string }`
/// - returns: `TaskDetail`
/// - errors: `TASK_NOT_FOUND`
#[tauri::command]
pub async fn get_task_detail(
    state: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
    task_id: String,
) -> Result<TaskDetail, IpcError> {
    let engine = state.lock().await;
    engine.get_task_detail(&task_id)
}

/// Opens a folder in the system file manager.
///
/// Corresponds to protocol: OpenFolderCommand
/// - name: `open_folder`
/// - params: `{ path: string }`
/// - returns: `true` on success
/// - errors: `PATH_NOT_ALLOWED` if path is outside storage root
#[tauri::command]
pub async fn open_folder(
    state: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
    path: String,
) -> Result<bool, IpcError> {
    let engine = state.lock().await;
    engine.open_folder(&path)
}

/// Deletes a task and its associated data from the database and filesystem.
///
/// Corresponds to protocol: DeleteTaskCommand
/// - name: `delete_task`
/// - params: `{ task_id: string }`
/// - returns: `true` on success
/// - errors: `TASK_NOT_FOUND`
#[tauri::command]
pub async fn delete_task(
    state: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
    task_id: String,
) -> Result<bool, IpcError> {
    let engine = state.lock().await;
    engine.delete_task(&task_id)
}

/// Returns the cover image of a task as base64 data URL (e.g., data:image/jpeg;base64,...).
#[tauri::command]
pub async fn get_cover_image(
    state: tauri::State<'_, tokio::sync::Mutex<StorageEngine>>,
    task_id: String,
) -> Result<String, IpcError> {
    use base64::Engine;
    use crate::models::ImageType;
    use std::io::Read;
    let engine = state.lock().await;
    let detail = engine.get_task_detail(&task_id)?;
    let folder = detail.task.folder_path.as_deref().unwrap_or("");
    // Find the cover image from the images list
    let cover_local = detail
        .images
        .iter()
        .find(|img| matches!(img.image_type, ImageType::Cover))
        .and_then(|img| img.local_path.as_deref());
    if folder.is_empty() || cover_local.is_none() {
        return Err(IpcError {
            code: crate::models::ErrorCode::TaskNotFound,
            message: "No cover image available".into(),
            recoverable: false,
            step: None,
            details: None,
        });
    }
    let cover_local = cover_local.unwrap();
    let path = std::path::PathBuf::from(folder).join(cover_local);
    let mut file = std::fs::File::open(&path).map_err(|e| IpcError {
        code: crate::models::ErrorCode::StorageFailed,
        message: format!("Failed to open cover: {}", e),
        recoverable: false,
        step: None,
        details: None,
    })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| IpcError {
        code: crate::models::ErrorCode::StorageFailed,
        message: format!("Failed to read cover: {}", e),
        recoverable: false,
        step: None,
        details: None,
    })?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    let ext = std::path::Path::new(cover_local)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    Ok(format!("data:image/{};base64,{}", ext, b64))
}
