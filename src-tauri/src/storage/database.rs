// EGrab - Storage Engine: Database Layer
// SQLite operations for tasks and images tables.
// Uses rusqlite with parameterized queries to prevent SQL injection.

use crate::models::{
    ErrorCode, ImageIndexInput, ImageRecord, ImageType, Task, TaskDetail, TaskFilter, TaskStatus,
    TaskSummary, TaskUpdate,
};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Manages SQLite database operations with thread-safe access
/// via the Mutex held by StorageEngine.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens (or creates) the SQLite database at `db_path` and runs schema migrations.
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        // Ensure the parent directory exists.
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {:?}", parent))?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path.as_ref()))?;

        // Enable WAL mode for better concurrent read performance.
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Runs schema.sql to ensure all tables and indexes exist.
    fn run_migrations(&self) -> Result<()> {
        let schema = include_str!("schema.sql");
        self.conn
            .execute_batch(schema)
            .context("Failed to run schema migrations")?;
        tracing::info!("Storage schema migrations completed successfully");
        Ok(())
    }

    // ---- Task Operations ----

    /// Inserts a new task record into the database.
    pub fn insert_task(&self, task: &Task) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO tasks (id, url, platform, item_id, title, status, created_at, folder_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    task.id,
                    task.url,
                    task.platform,
                    task.item_id,
                    task.title,
                    task.status.to_string(),
                    task.created_at,
                    task.folder_path,
                ],
            )
            .context("Failed to insert task")?;
        Ok(())
    }

    /// Checks whether a task with the given (platform, item_id) exists
    /// and is in a non-failed/cancelled state. Returns the existing task id if found.
    pub fn check_duplicate(&self, platform: &str, item_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM tasks
                 WHERE platform = ?1 AND item_id = ?2
                   AND status NOT IN ('failed', 'cancelled')
                 LIMIT 1",
            )
            .context("Failed to prepare duplicate check query")?;

        let result = stmt
            .query_row(params![platform, item_id], |row| row.get::<_, String>(0))
            .ok();

        Ok(result)
    }

    /// Replaces an existing task in a transaction (for force-rescrape scenario).
    /// Deletes the old task and its images, then inserts the new task.
    pub fn replace_task(&self, old_task_id: &str, new_task: &Task) -> Result<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to begin transaction for task replacement")?;

        tx.execute(
            "DELETE FROM images WHERE task_id = ?1",
            params![old_task_id],
        )
        .context("Failed to delete old images")?;

        tx.execute("DELETE FROM tasks WHERE id = ?1", params![old_task_id])
            .context("Failed to delete old task")?;

        tx.execute(
            "INSERT INTO tasks (id, url, platform, item_id, title, status, created_at, folder_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                new_task.id,
                new_task.url,
                new_task.platform,
                new_task.item_id,
                new_task.title,
                new_task.status.to_string(),
                new_task.created_at,
                new_task.folder_path,
            ],
        )
        .context("Failed to insert replacement task")?;

        tx.commit()
            .context("Failed to commit task replacement transaction")?;
        Ok(())
    }

    /// Deletes a task and its associated images from the database.
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .context("Failed to begin transaction for task deletion")?;

        tx.execute("DELETE FROM images WHERE task_id = ?1", params![task_id])
            .context("Failed to delete task images")?;

        tx.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
            .context("Failed to delete task")?;

        tx.commit()
            .context("Failed to commit task deletion transaction")?;
        Ok(())
    }

    /// Updates selected fields of a task record.
    pub fn update_task(&self, task_id: &str, updates: &TaskUpdate) -> Result<()> {
        let mut set_clauses: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref status) = updates.status {
            set_clauses.push("status = ?".to_string());
            param_values.push(Box::new(status.to_string()));
        }
        if let Some(ref title) = updates.title {
            set_clauses.push("title = ?".to_string());
            param_values.push(Box::new(title.clone()));
        }
        if let Some(ref folder_path) = updates.folder_path {
            set_clauses.push("folder_path = ?".to_string());
            param_values.push(Box::new(folder_path.clone()));
        }

        if set_clauses.is_empty() {
            return Ok(()); // Nothing to update.
        }

        let sql = format!("UPDATE tasks SET {} WHERE id = ?", set_clauses.join(", "));

        // Add the task_id as the last parameter.
        param_values.push(Box::new(task_id.to_string()));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        self.conn
            .execute(&sql, param_refs.as_slice())
            .context("Failed to execute update statement")?;

        Ok(())
    }

    /// Returns the task id (String) if a duplicate exists. Used by replace logic.
    pub fn get_duplicate_task_info(
        &self,
        platform: &str,
        item_id: &str,
    ) -> Result<Option<(String, Option<String>)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, folder_path FROM tasks
                 WHERE platform = ?1 AND item_id = ?2
                   AND status NOT IN ('failed', 'cancelled')
                 LIMIT 1",
            )
            .context("Failed to prepare duplicate info query")?;

        let result = stmt
            .query_row(params![platform, item_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .ok();

        Ok(result)
    }

    /// Fetches a single task by id.
    pub fn get_task(&self, task_id: &str) -> Result<Option<Task>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, url, platform, item_id, title, status, created_at, folder_path
                 FROM tasks WHERE id = ?1",
            )
            .context("Failed to prepare get_task query")?;

        let result = stmt
            .query_row(params![task_id], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    platform: row.get(2)?,
                    item_id: row.get(3)?,
                    title: row.get(4)?,
                    status: TaskStatus::from_str(&row.get::<_, String>(5)?),
                    created_at: row.get(6)?,
                    folder_path: row.get(7)?,
                })
            })
            .ok();

        Ok(result)
    }

    /// Queries tasks with optional filters. Returns TaskSummary list.
    pub fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskSummary>> {
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref platform) = filter.platform {
            conditions.push("t.platform = ?".to_string());
            param_values.push(Box::new(platform.clone()));
        }
        if let Some(ref status) = filter.status {
            conditions.push("t.status = ?".to_string());
            param_values.push(Box::new(status.to_string()));
        }
        if let Some(ref keyword) = filter.keyword {
            if !keyword.is_empty() {
                conditions.push("t.title LIKE ?".to_string());
                param_values.push(Box::new(format!("%{}%", keyword)));
            }
        }
        if let Some(ref item_id) = filter.item_id {
            conditions.push("t.item_id = ?".to_string());
            param_values.push(Box::new(item_id.clone()));
        }
        if let Some(ref start_time) = filter.start_time {
            conditions.push("t.created_at >= ?".to_string());
            param_values.push(Box::new(start_time.clone()));
        }
        if let Some(ref end_time) = filter.end_time {
            conditions.push("t.created_at <= ?".to_string());
            param_values.push(Box::new(end_time.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit = filter.limit.unwrap_or(50).min(200);
        let offset = filter.offset.unwrap_or(0);

        let sql = format!(
            "SELECT t.id, t.url, t.platform, t.item_id, t.title, t.status, t.created_at, t.folder_path,
                    (SELECT i.local_path FROM images i WHERE i.task_id = t.id AND i.type = 'cover' LIMIT 1) as cover_path
             FROM tasks t
             {}
             ORDER BY t.created_at DESC
             LIMIT ? OFFSET ?",
            where_clause
        );

        // Build parameter references for the query.
        let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = param_values;
        all_params.push(Box::new(limit));
        all_params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            all_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self
            .conn
            .prepare(&sql)
            .context("Failed to prepare query_tasks statement")?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let status_str: String = row.get(5)?;
                Ok(TaskSummary {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    platform: row.get(2)?,
                    item_id: row.get(3)?,
                    title: row.get(4)?,
                    status: TaskStatus::from_str(&status_str),
                    created_at: row.get(6)?,
                    folder_path: row.get(7)?,
                    cover_path: row.get(8)?,
                })
            })
            .context("Failed to execute query_tasks")?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.context("Failed to read task row")?);
        }

        Ok(results)
    }

    // ---- Image Operations ----

    /// Inserts an image record and returns the assigned rowid.
    pub fn insert_image(&self, image: &ImageIndexInput) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO images (task_id, type, original_url, local_path, width, height, size_bytes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    image.task_id,
                    image.image_type.to_string(),
                    image.original_url,
                    image.local_path,
                    image.width,
                    image.height,
                    image.size_bytes.map(|v| v as i64),
                ],
            )
            .context("Failed to insert image record")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Returns all image records for a given task.
    pub fn get_images_for_task(&self, task_id: &str) -> Result<Vec<ImageRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, task_id, type, original_url, local_path, width, height, size_bytes
                 FROM images WHERE task_id = ?1
                 ORDER BY id",
            )
            .context("Failed to prepare get_images query")?;

        let rows = stmt
            .query_map(params![task_id], |row| {
                let type_str: String = row.get(2)?;
                let size_bytes: Option<i64> = row.get(7)?;
                Ok(ImageRecord {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    image_type: ImageType::from_str(&type_str),
                    original_url: row.get(3)?,
                    local_path: row.get(4)?,
                    width: row.get(5)?,
                    height: row.get(6)?,
                    size_bytes: size_bytes.map(|v| v as u64),
                })
            })
            .context("Failed to query images")?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}

// ======== String conversion helpers for enums ========

impl TaskStatus {
    /// Converts enum variant to lowercase string matching DB values.
    pub fn to_string(&self) -> String {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Success => "success",
            TaskStatus::Failed => "failed",
            TaskStatus::Partial => "partial",
            TaskStatus::Cancelled => "cancelled",
        }
        .to_string()
    }

    /// Parses from DB string to enum variant.
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => TaskStatus::Pending,
            "running" => TaskStatus::Running,
            "success" => TaskStatus::Success,
            "failed" => TaskStatus::Failed,
            "partial" => TaskStatus::Partial,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Failed, // Unknown status treated as Failed for safety.
        }
    }
}

impl ImageType {
    /// Converts enum variant to lowercase string matching DB values.
    pub fn to_string(&self) -> String {
        match self {
            ImageType::Cover => "cover",
            ImageType::Gallery => "gallery",
            ImageType::Detail => "detail",
            ImageType::Sku => "sku",
        }
        .to_string()
    }

    /// Parses from DB string to enum variant.
    pub fn from_str(s: &str) -> Self {
        match s {
            "cover" => ImageType::Cover,
            "gallery" => ImageType::Gallery,
            "detail" => ImageType::Detail,
            "sku" => ImageType::Sku,
            _ => ImageType::Cover, // Fallback; should not happen with valid data.
        }
    }
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        // Use in-memory database for tests.
        Database {
            conn: Connection::open_in_memory().unwrap(),
        }
    }

    fn make_task(id: &str, platform: &str, item_id: &str) -> Task {
        Task {
            id: id.to_string(),
            url: format!("https://item.{}.com/item.htm?id={}", platform, item_id),
            platform: platform.to_string(),
            item_id: item_id.to_string(),
            title: "Test Product".to_string(),
            status: TaskStatus::Pending,
            created_at: "2026-05-10T00:00:00Z".to_string(),
            folder_path: Some(format!("/tmp/egrab/{}_{}_20260510/", platform, item_id)),
        }
    }

    #[test]
    fn test_open_in_memory() {
        let db = test_db();
        db.run_migrations().unwrap();
    }

    #[test]
    fn test_insert_and_get_task() {
        let db = test_db();
        db.run_migrations().unwrap();

        let task = make_task("task-1", "taobao", "12345");
        db.insert_task(&task).unwrap();

        let fetched = db.get_task("task-1").unwrap().unwrap();
        assert_eq!(fetched.id, "task-1");
        assert_eq!(fetched.platform, "taobao");
        assert_eq!(fetched.item_id, "12345");
        assert_eq!(fetched.title, "Test Product");
    }

    #[test]
    fn test_check_duplicate_found() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "12345"))
            .unwrap();

        let dup = db.check_duplicate("taobao", "12345").unwrap();
        assert_eq!(dup, Some("task-1".to_string()));
    }

    #[test]
    fn test_check_duplicate_not_found() {
        let db = test_db();
        db.run_migrations().unwrap();

        let dup = db.check_duplicate("taobao", "99999").unwrap();
        assert_eq!(dup, None);
    }

    #[test]
    fn test_check_duplicate_skips_failed() {
        let db = test_db();
        db.run_migrations().unwrap();

        let mut task = make_task("task-fail", "taobao", "12345");
        task.status = TaskStatus::Failed;
        db.insert_task(&task).unwrap();

        let dup = db.check_duplicate("taobao", "12345").unwrap();
        assert_eq!(dup, None);
    }

    #[test]
    fn test_update_task() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "12345"))
            .unwrap();

        let updates = TaskUpdate {
            status: Some(TaskStatus::Running),
            title: Some("Updated Title".to_string()),
            folder_path: None,
        };
        db.update_task("task-1", &updates).unwrap();

        let fetched = db.get_task("task-1").unwrap().unwrap();
        assert!(matches!(fetched.status, TaskStatus::Running));
        assert_eq!(fetched.title, "Updated Title");
    }

    #[test]
    fn test_insert_and_query_images() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "12345"))
            .unwrap();

        let img = ImageIndexInput {
            task_id: "task-1".to_string(),
            image_type: ImageType::Cover,
            original_url: "https://img.example.com/cover.jpg".to_string(),
            local_path: Some("cover/cover_001.jpg".to_string()),
            width: Some(800),
            height: Some(600),
            size_bytes: Some(12345),
        };
        db.insert_image(&img).unwrap();

        let images = db.get_images_for_task("task-1").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].original_url, "https://img.example.com/cover.jpg");
        assert!(matches!(images[0].image_type, ImageType::Cover));
    }

    #[test]
    fn test_query_tasks_basic() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "111"))
            .unwrap();
        db.insert_task(&make_task("task-2", "jd", "222")).unwrap();

        let filter = TaskFilter {
            platform: None,
            status: None,
            keyword: None,
            item_id: None,
            start_time: None,
            end_time: None,
            limit: Some(10),
            offset: Some(0),
        };
        let results = db.query_tasks(&filter).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_tasks_filter_by_platform() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "111"))
            .unwrap();
        db.insert_task(&make_task("task-2", "jd", "222")).unwrap();

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
        let results = db.query_tasks(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].platform, "taobao");
    }

    #[test]
    fn test_query_tasks_filter_by_keyword() {
        let db = test_db();
        db.run_migrations().unwrap();

        let mut task_a = make_task("task-a", "taobao", "111");
        task_a.title = "Apple iPhone".to_string();
        db.insert_task(&task_a).unwrap();

        let mut task_b = make_task("task-b", "jd", "222");
        task_b.title = "Samsung Galaxy".to_string();
        db.insert_task(&task_b).unwrap();

        let filter = TaskFilter {
            platform: None,
            status: None,
            keyword: Some("iphone".to_string()),
            item_id: None,
            start_time: None,
            end_time: None,
            limit: Some(10),
            offset: Some(0),
        };
        let results = db.query_tasks(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Apple iPhone");
    }

    #[test]
    fn test_query_tasks_cover_path() {
        let db = test_db();
        db.run_migrations().unwrap();

        db.insert_task(&make_task("task-1", "taobao", "111"))
            .unwrap();

        let img = ImageIndexInput {
            task_id: "task-1".to_string(),
            image_type: ImageType::Cover,
            original_url: "https://img.example.com/cover.jpg".to_string(),
            local_path: Some("cover/cover_001.jpg".to_string()),
            width: Some(800),
            height: Some(600),
            size_bytes: None,
        };
        db.insert_image(&img).unwrap();

        let filter = TaskFilter {
            platform: None,
            status: None,
            keyword: None,
            item_id: None,
            start_time: None,
            end_time: None,
            limit: Some(10),
            offset: Some(0),
        };
        let results = db.query_tasks(&filter).unwrap();
        assert_eq!(
            results[0].cover_path,
            Some("cover/cover_001.jpg".to_string())
        );
    }
}
