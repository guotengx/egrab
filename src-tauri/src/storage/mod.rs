// EGrab - Storage Engine Module
// Module entry point. Coordinates database and filesystem operations.
// Derived from: src/protocols/storage.ts, docs/protocols/storage-interface.md

pub mod database;
pub mod filesystem;

use crate::models::{
    DuplicateTaskConflict, ErrorCode, ImageIndexInput, IpcError, MetaJsonDocument, ProductData,
    RawJsonDocument, ScrapeErrorInfo, Task, TaskDetail, TaskFilter, TaskStatus, TaskSummary,
    TaskUpdate,
};
use anyhow::Result;
use database::Database;
use std::path::PathBuf;
use std::sync::Mutex;

/// The core storage engine, combining SQLite indexing and filesystem data management.
/// Thread-safe via Mutex-protected database connection.
pub struct StorageEngine {
    db: Mutex<Option<Database>>,
    storage_root: String,
}

impl StorageEngine {
    /// Creates a new StorageEngine.
    /// `init()` must be called before any other operation that accesses the database.
    pub fn new(storage_root: String) -> Self {
        Self {
            db: Mutex::new(None),
            storage_root,
        }
    }

    // ---- Public API (matches StorageEngine protocol interface) ----

    /// Initializes the storage engine: opens/creates the SQLite database and runs migrations.
    pub fn init(&self) -> Result<()> {
        let db_path = filesystem::database_path();
        let db = Database::open(&db_path)?;

        // Ensure storage root directory exists.
        std::fs::create_dir_all(&self.storage_root)?;

        let mut guard = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;
        *guard = Some(db);

        tracing::info!(
            "StorageEngine initialized: db={:?}, storage_root={}",
            db_path,
            self.storage_root
        );
        Ok(())
    }

    /// Acquires a lock on the database, returning a reference.
    /// Returns an error if not initialized or lock is poisoned.
    fn with_db<F, T>(&self, f: F) -> Result<T, IpcError>
    where
        F: FnOnce(&Database) -> Result<T, IpcError>,
    {
        let guard = self.db.lock().map_err(|e| IpcError {
            code: ErrorCode::StorageFailed,
            message: format!("Database lock error: {}", e),
            recoverable: false,
            step: None,
            details: None,
        })?;

        let db = guard.as_ref().ok_or_else(|| IpcError {
            code: ErrorCode::StorageFailed,
            message: "StorageEngine not initialized. Call init() first.".to_string(),
            recoverable: false,
            step: None,
            details: None,
        })?;

        f(db)
    }

    /// Creates a new task, checking for duplicates unless force=true.
    /// Returns the created Task on success, or a DuplicateTaskConflict error.
    pub fn create_task(
        &self,
        url: &str,
        platform: &str,
        item_id: &str,
        force: Option<bool>,
    ) -> Result<Task, IpcError> {
        let force = force.unwrap_or(false);

        self.with_db(|db| {
            // Check for duplicates unless force=true.
            if !force {
                if let Some(existing_id) = db
                    .check_duplicate(platform, item_id)
                    .map_err(|e| storage_error(&e))?
                {
                    let (_, folder_path) = db
                        .get_duplicate_task_info(platform, item_id)
                        .map_err(|e| storage_error(&e))?
                        .unwrap_or((existing_id.clone(), None));

                    return Err(IpcError {
                        code: ErrorCode::DuplicateTask,
                        message: format!(
                            "Task already exists for {}/{}: {}",
                            platform, item_id, existing_id
                        ),
                        recoverable: true,
                        step: None,
                        details: Some(
                            serde_json::to_value(DuplicateTaskConflict::new(
                                existing_id,
                                folder_path,
                            ))
                            .unwrap_or_default(),
                        ),
                    });
                }
            }

            // If force, clean up old task data.
            if force {
                if let Some((_old_id, old_folder)) = db
                    .get_duplicate_task_info(platform, item_id)
                    .map_err(|e| storage_error(&e))?
                {
                    if let Some(ref path) = old_folder {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            std::fs::remove_dir_all(&p).ok();
                        }
                    }
                }
            }

            Ok(())
        })?;

        // Insert the new task (this also creates the folder).
        self.with_db(|db| self.insert_new_task(db, url, platform, item_id))
    }

    /// Updates a task's mutable fields.
    pub fn update_task(&self, task_id: &str, updates: TaskUpdate) -> Result<(), IpcError> {
        self.with_db(|db| {
            db.update_task(task_id, &updates)
                .map_err(|e| storage_error(&e))
        })
    }

    /// Saves product metadata as meta.json in the task folder.
    /// Returns the absolute path to the saved file.
    pub fn save_meta(&self, task_id: &str, product: &ProductData) -> Result<String, IpcError> {
        self.with_db(|db| {
            let task = db
                .get_task(task_id)
                .map_err(|e| storage_error(&e))?
                .ok_or_else(|| IpcError {
                    code: ErrorCode::TaskNotFound,
                    message: format!("Task not found: {}", task_id),
                    recoverable: false,
                    step: None,
                    details: None,
                })?;

            let folder_path = task.folder_path.clone().ok_or_else(|| IpcError {
                code: ErrorCode::StorageFailed,
                message: "Task has no folder_path".to_string(),
                recoverable: false,
                step: None,
                details: None,
            })?;

            let doc = filesystem::build_meta_document(&task, product);
            let path = filesystem::write_meta_json(&PathBuf::from(&folder_path), &doc)
                .map_err(|e| storage_error(&e))?;

            Ok(path.to_string_lossy().to_string())
        })
    }

    /// Builds a MetaJsonDocument from task metadata and product data.
    pub fn build_meta_document(
        &self,
        task_id: &str,
        product: &ProductData,
    ) -> Result<MetaJsonDocument, IpcError> {
        self.with_db(|db| {
            let task = db
                .get_task(task_id)
                .map_err(|e| storage_error(&e))?
                .ok_or_else(|| IpcError {
                    code: ErrorCode::TaskNotFound,
                    message: format!("Task not found: {}", task_id),
                    recoverable: false,
                    step: None,
                    details: None,
                })?;

            Ok(filesystem::build_meta_document(&task, product))
        })
    }

    /// Saves raw scrape data as raw.json in the task folder.
    pub fn save_raw(
        &self,
        task_id: &str,
        raw_data: &serde_json::Value,
        parser_errors: &[ScrapeErrorInfo],
    ) -> Result<String, IpcError> {
        self.with_db(|db| {
            let task = db
                .get_task(task_id)
                .map_err(|e| storage_error(&e))?
                .ok_or_else(|| IpcError {
                    code: ErrorCode::TaskNotFound,
                    message: format!("Task not found: {}", task_id),
                    recoverable: false,
                    step: None,
                    details: None,
                })?;

            let folder_path = task.folder_path.clone().ok_or_else(|| IpcError {
                code: ErrorCode::StorageFailed,
                message: "Task has no folder_path".to_string(),
                recoverable: false,
                step: None,
                details: None,
            })?;

            let path = filesystem::write_raw_json(
                &PathBuf::from(&folder_path),
                &task.url,
                &task.platform,
                &task.item_id,
                raw_data,
                parser_errors,
            )
            .map_err(|e| storage_error(&e))?;

            Ok(path.to_string_lossy().to_string())
        })
    }

    /// Builds a RawJsonDocument.
    pub fn build_raw_document(
        &self,
        task_id: &str,
        raw_data: &serde_json::Value,
        parser_errors: &[ScrapeErrorInfo],
    ) -> Result<RawJsonDocument, IpcError> {
        self.with_db(|db| {
            let task = db
                .get_task(task_id)
                .map_err(|e| storage_error(&e))?
                .ok_or_else(|| IpcError {
                    code: ErrorCode::TaskNotFound,
                    message: format!("Task not found: {}", task_id),
                    recoverable: false,
                    step: None,
                    details: None,
                })?;

            Ok(RawJsonDocument {
                version: "1.0.0".to_string(),
                platform: task.platform.clone(),
                item_id: task.item_id.clone(),
                scraped_at: chrono_now_str(),
                url: task.url.clone(),
                raw_data: raw_data.clone(),
                parser_errors: parser_errors.to_vec(),
            })
        })
    }

    /// Indexes a downloaded image in the SQLite database.
    pub fn index_image(&self, image: ImageIndexInput) -> Result<(), IpcError> {
        tracing::info!(
            "index_image: type={}, local_path={:?}, task_id={}",
            image.image_type.to_string(),
            image.local_path,
            image.task_id
        );
        self.with_db(|db| {
            db.insert_image(&image).map_err(|e| storage_error(&e))?;
            Ok(())
        })
    }

    /// Queries tasks with optional filters. Returns TaskSummary list.
    pub fn query_tasks(&self, filter: TaskFilter) -> Result<Vec<TaskSummary>, IpcError> {
        self.with_db(|db| db.query_tasks(&filter).map_err(|e| storage_error(&e)))
    }

    /// Returns detailed information about a task, including product data and images.
    pub fn get_task_detail(&self, task_id: &str) -> Result<TaskDetail, IpcError> {
        self.with_db(|db| {
            let task = db
                .get_task(task_id)
                .map_err(|e| storage_error(&e))?
                .ok_or_else(|| IpcError {
                    code: ErrorCode::TaskNotFound,
                    message: format!("Task not found: {}", task_id),
                    recoverable: false,
                    step: None,
                    details: None,
                })?;

            let images = db
                .get_images_for_task(task_id)
                .map_err(|e| storage_error(&e))?;

            let (product, meta_path) = if let Some(ref fp) = task.folder_path {
                let folder = PathBuf::from(fp);
                let meta_file = folder.join("meta.json");
                let meta_path = if meta_file.exists() {
                    Some(meta_file.to_string_lossy().to_string())
                } else {
                    None
                };
                let product = filesystem::read_meta_json(&folder).unwrap_or(None);
                (product, meta_path)
            } else {
                (None, None)
            };

            let raw_path = task
                .folder_path
                .as_ref()
                .and_then(|fp| filesystem::raw_json_path(&PathBuf::from(fp)))
                .map(|p| p.to_string_lossy().to_string());

            let errors: Vec<ScrapeErrorInfo> = images
                .iter()
                .filter(|img| img.local_path.is_none())
                .map(|img| ScrapeErrorInfo {
                    step: crate::models::ScrapeStep::Downloading,
                    code: "IMAGE_DOWNLOAD_FAILED".to_string(),
                    message: format!("Image download failed: {}", img.original_url),
                    recoverable: true,
                })
                .collect();

            Ok(TaskDetail {
                task,
                product,
                images,
                raw_path,
                meta_path,
                errors,
            })
        })
    }

    /// Deletes a task and its associated data from the filesystem and database.
    ///
    /// Folder deletion happens first so that if it fails, the database record
    /// remains intact and can be retried. The database deletion runs in a
    /// transaction that removes both the task row and its image records.
    pub fn delete_task(&self, task_id: &str) -> Result<bool, IpcError> {
        // 1. Get task detail to obtain the folder_path.
        let detail = self.get_task_detail(task_id)?;

        // 2. Delete folder from filesystem before touching the database.
        if let Some(ref folder_path) = detail.task.folder_path {
            if !folder_path.is_empty() {
                let p = PathBuf::from(folder_path);
                if p.exists() {
                    std::fs::remove_dir_all(&p).map_err(|e| IpcError {
                        code: ErrorCode::StorageFailed,
                        message: format!("Failed to delete folder: {}", e),
                        recoverable: false,
                        step: None,
                        details: None,
                    })?;
                }
            }
        }

        // 3. Delete database records (task + images in a transaction).
        self.with_db(|db| db.delete_task(task_id).map_err(|e| storage_error(&e)))?;

        tracing::info!("Deleted task: {}", task_id);
        Ok(true)
    }

    /// Checks if a duplicate task exists. Returns the task id if found.
    pub fn check_duplicate(
        &self,
        platform: &str,
        item_id: &str,
    ) -> Result<Option<String>, IpcError> {
        self.with_db(|db| {
            db.check_duplicate(platform, item_id)
                .map_err(|e| storage_error(&e))
        })
    }

    /// Opens a folder in the system file manager, with path safety validation.
    pub fn open_folder(&self, path: &str) -> Result<bool, IpcError> {
        filesystem::open_folder(path, &self.storage_root).map_err(|e| IpcError {
            code: ErrorCode::PathNotAllowed,
            message: format!("Cannot open folder: {}", e),
            recoverable: true,
            step: None,
            details: None,
        })
    }

    // ---- Internal Helpers ----

    /// Creates the file system folder and inserts the task record.
    fn insert_new_task(
        &self,
        db: &Database,
        url: &str,
        platform: &str,
        item_id: &str,
    ) -> Result<Task, IpcError> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono_now_str().replace('-', "").replace(':', "");
        // Take first 15 chars: YYYYMMDDTHHMMSS
        let ts_short = if timestamp.len() >= 15 {
            &timestamp[..15]
        } else {
            &timestamp
        };

        let folder_path =
            filesystem::create_task_folder(&self.storage_root, platform, item_id, ts_short)
                .map_err(|e| storage_error(&e))?;

        let task = Task {
            id: task_id,
            url: url.to_string(),
            platform: platform.to_string(),
            item_id: item_id.to_string(),
            title: String::new(),
            status: TaskStatus::Pending,
            created_at: chrono_now_str(),
            folder_path: Some(folder_path.to_string_lossy().to_string()),
        };

        db.insert_task(&task).map_err(|e| storage_error(&e))?;

        tracing::info!(
            "Created task: id={}, platform={}, item_id={}",
            task.id,
            task.platform,
            task.item_id
        );
        Ok(task)
    }
}

// ---- Utility Helpers ----

/// Converts anyhow error to IpcError with StorageFailed code.
fn storage_error(e: &anyhow::Error) -> IpcError {
    IpcError {
        code: ErrorCode::StorageFailed,
        message: format!("Storage error: {}", e),
        recoverable: false,
        step: None,
        details: None,
    }
}

/// Returns a simple ISO 8601 timestamp for the current time.
fn chrono_now_str() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = days_to_date(days_since_epoch as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    let mut year: i64 = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_lengths: [i64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: u32 = 1;
    for &ml in &month_lengths {
        if days < ml {
            break;
        }
        days -= ml;
        month += 1;
    }

    (year, month, (days + 1) as u32)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_storage() -> (StorageEngine, TempDir) {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_str().unwrap().to_string();
        let engine = StorageEngine::new(root);
        // Initialize with test mode: set up database directly.
        {
            let mut guard = engine.db.lock().unwrap();
            *guard = Some(Database::open(":memory:").unwrap());
        }
        (engine, dir)
    }

    #[test]
    fn test_create_and_query_task() {
        let (engine, _dir) = make_test_storage();
        let task = engine
            .create_task(
                "https://item.taobao.com/item.htm?id=123",
                "taobao",
                "123",
                None,
            )
            .unwrap();

        assert_eq!(task.platform, "taobao");
        assert_eq!(task.item_id, "123");
        assert!(matches!(task.status, TaskStatus::Pending));
        assert!(task.folder_path.is_some());

        let filter = TaskFilter {
            platform: Some("taobao".to_string()),
            status: None,
            keyword: None,
            item_id: None,
            start_time: None,
            end_time: None,
            limit: Some(10),
            offset: Some(0),
        };
        let results = engine.query_tasks(filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, task.id);
    }

    #[test]
    fn test_duplicate_detection() {
        let (engine, _dir) = make_test_storage();
        engine
            .create_task(
                "https://item.taobao.com/item.htm?id=123",
                "taobao",
                "123",
                None,
            )
            .unwrap();

        let result = engine.create_task(
            "https://item.taobao.com/item.htm?id=123",
            "taobao",
            "123",
            None,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::DuplicateTask));
    }

    #[test]
    fn test_force_rescrape_allows_duplicate() {
        let (engine, _dir) = make_test_storage();
        engine
            .create_task(
                "https://item.taobao.com/item.htm?id=123",
                "taobao",
                "123",
                None,
            )
            .unwrap();

        let result = engine.create_task(
            "https://item.taobao.com/item.htm?id=123",
            "taobao",
            "123",
            Some(true),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_task_status() {
        let (engine, _dir) = make_test_storage();
        let task = engine
            .create_task(
                "https://item.taobao.com/item.htm?id=123",
                "taobao",
                "123",
                None,
            )
            .unwrap();

        engine
            .update_task(
                &task.id,
                TaskUpdate {
                    status: Some(TaskStatus::Running),
                    title: Some("Running Task".to_string()),
                    folder_path: None,
                },
            )
            .unwrap();

        let detail = engine.get_task_detail(&task.id).unwrap();
        assert!(matches!(detail.task.status, TaskStatus::Running));
        assert_eq!(detail.task.title, "Running Task");
    }

    #[test]
    fn test_save_meta() {
        let (engine, _dir) = make_test_storage();
        let task = engine
            .create_task(
                "https://item.taobao.com/item.htm?id=123",
                "taobao",
                "123",
                None,
            )
            .unwrap();

        let product = ProductData {
            title: "Test Product".to_string(),
            cover: crate::models::ImageRef {
                original_url: "".to_string(),
                thumbnail_url: "".to_string(),
                local_path: None,
            },
            gallery: vec![],
            description: crate::models::Description {
                text: "".to_string(),
                html: None,
                specs: vec![],
            },
            detail_images: vec![],
            skus: vec![],
            sku_images: std::collections::HashMap::new(),
            price: crate::models::PriceRange {
                min_price: 0.0,
                max_price: 0.0,
                currency: "CNY".to_string(),
            },
            shop: crate::models::ShopInfo {
                name: "".to_string(),
                url: "".to_string(),
            },
        };

        let meta_path = engine.save_meta(&task.id, &product).unwrap();
        assert!(meta_path.ends_with("meta.json"));

        let detail = engine.get_task_detail(&task.id).unwrap();
        assert!(detail.meta_path.is_some());
        assert!(detail.product.is_some());
        assert_eq!(detail.product.unwrap().title, "Test Product");
    }

    #[test]
    fn test_get_task_detail_not_found() {
        let (engine, _dir) = make_test_storage();
        let result = engine.get_task_detail("nonexistent-id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::TaskNotFound));
    }
}
