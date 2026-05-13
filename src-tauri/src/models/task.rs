// EGrab - Core Data Models: Task
// Derived from: docs/protocols/data-models.md v1.0.0

use serde::{Deserialize, Serialize};

use super::product::ProductData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    pub id: String,
    pub url: String,
    pub platform: String,
    pub item_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: String,
    pub folder_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
    Partial,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageRecord {
    pub id: i64,
    pub task_id: String,
    #[serde(rename = "type")]
    pub image_type: ImageType,
    pub original_url: String,
    pub local_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    Cover,
    Gallery,
    Detail,
    Sku,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskFilter {
    pub platform: Option<String>,
    pub status: Option<TaskStatus>,
    pub keyword: Option<String>,
    pub item_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskSummary {
    pub id: String,
    pub url: String,
    pub platform: String,
    pub item_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: String,
    pub folder_path: Option<String>,
    pub cover_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskDetail {
    pub task: Task,
    pub product: Option<ProductData>,
    pub images: Vec<ImageRecord>,
    pub raw_path: Option<String>,
    pub meta_path: Option<String>,
    pub errors: Vec<ScrapeErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub folder_path: Option<String>,
    pub product: Option<ProductData>,
    pub image_total: u32,
    pub image_success: u32,
    pub image_failed: u32,
    pub errors: Vec<ScrapeErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScrapeErrorInfo {
    pub step: ScrapeStep,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrapeStep {
    Connecting,
    PageLoading,
    Parsing,
    Downloading,
    Saving,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    CdpConnectFailed,
    CdpTimeout,
    CdpLaunchTimeout,
    NoBrowserFound,
    UrlInvalid,
    UnsupportedPlatform,
    ItemIdExtractFailed,
    DuplicateTask,
    TaskAlreadyRunning,
    TaskNotFound,
    TaskCancelled,
    ParseFailed,
    ImageDownloadFailed,
    StorageFailed,
    PathNotAllowed,
    ConfigInvalid,
    UnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub step: Option<ScrapeStep>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskUpdate {
    pub status: Option<TaskStatus>,
    pub title: Option<String>,
    pub folder_path: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DuplicateTaskConflict {
    pub existing_task_id: String,
    pub existing_folder_path: Option<String>,
    pub code: ErrorCode,
}

impl DuplicateTaskConflict {
    pub fn new(existing_task_id: String, existing_folder_path: Option<String>) -> Self {
        Self {
            existing_task_id,
            existing_folder_path,
            code: ErrorCode::DuplicateTask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageIndexInput {
    pub task_id: String,
    #[serde(rename = "type")]
    pub image_type: ImageType,
    pub original_url: String,
    pub local_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_serializes_to_screaming_snake_case() {
        let json = serde_json::to_string(&ErrorCode::CdpConnectFailed).unwrap();
        assert_eq!(json, "\"CDP_CONNECT_FAILED\"");

        let json = serde_json::to_string(&ErrorCode::DuplicateTask).unwrap();
        assert_eq!(json, "\"DUPLICATE_TASK\"");

        let json = serde_json::to_string(&ErrorCode::UnknownError).unwrap();
        assert_eq!(json, "\"UNKNOWN_ERROR\"");

        let json = serde_json::to_string(&ErrorCode::ImageDownloadFailed).unwrap();
        assert_eq!(json, "\"IMAGE_DOWNLOAD_FAILED\"");

        let json = serde_json::to_string(&ErrorCode::CdpLaunchTimeout).unwrap();
        assert_eq!(json, "\"CDP_LAUNCH_TIMEOUT\"");

        let json = serde_json::to_string(&ErrorCode::NoBrowserFound).unwrap();
        assert_eq!(json, "\"NO_BROWSER_FOUND\"");
    }

    #[test]
    fn test_error_code_deserializes_from_screaming_snake_case() {
        let ec: ErrorCode = serde_json::from_str("\"CDP_CONNECT_FAILED\"").unwrap();
        assert!(matches!(ec, ErrorCode::CdpConnectFailed));

        let ec: ErrorCode = serde_json::from_str("\"DUPLICATE_TASK\"").unwrap();
        assert!(matches!(ec, ErrorCode::DuplicateTask));
    }

    #[test]
    fn test_ipc_error_code_is_error_code_enum() {
        let ipc_err = IpcError {
            code: ErrorCode::CdpTimeout,
            message: "Connection timed out".to_string(),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        };
        let json = serde_json::to_string(&ipc_err).unwrap();
        assert!(
            json.contains("\"CDP_TIMEOUT\""),
            "IpcError.code should serialize as SCREAMING_SNAKE_CASE, got: {}",
            json
        );

        // Verify round-trip
        let de: IpcError = serde_json::from_str(&json).unwrap();
        assert!(matches!(de.code, ErrorCode::CdpTimeout));
    }

    #[test]
    fn test_duplicate_task_conflict_code_is_fixed() {
        let conflict =
            DuplicateTaskConflict::new("task_123".to_string(), Some("/path".to_string()));
        assert!(matches!(conflict.code, ErrorCode::DuplicateTask));

        let json = serde_json::to_string(&conflict).unwrap();
        assert!(
            json.contains("\"DUPLICATE_TASK\""),
            "DuplicateTaskConflict.code should be DUPLICATE_TASK, got: {}",
            json
        );
    }
}
