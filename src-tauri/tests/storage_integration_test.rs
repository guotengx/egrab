// EGrab - Integration Test: Storage Engine CRUD
// Verifies P6-2: StorageEngine complete CRUD flow via Database public API.
// Each test uses an independent in-memory SQLite database for isolation.

use egrab::models::{
    ImageIndexInput, ImageRecord, ImageType, Task, TaskFilter, TaskStatus, TaskUpdate,
};
use egrab::storage::database::Database;

/// Helper: creates a fresh in-memory database with migrations applied.
fn test_db() -> Database {
    Database::open(":memory:").expect("Failed to open in-memory database")
}

/// Helper: builds a Task with deterministic fields for integration tests.
fn make_task(id: &str, platform: &str, item_id: &str, title: &str, status: TaskStatus) -> Task {
    Task {
        id: id.to_string(),
        url: format!("https://item.{}.com/item.htm?id={}", platform, item_id),
        platform: platform.to_string(),
        item_id: item_id.to_string(),
        title: title.to_string(),
        status,
        created_at: "2026-05-10T00:00:00Z".to_string(),
        folder_path: Some(format!("/tmp/egrab/{}_{}_{}/", platform, item_id, id)),
    }
}

// ---------------------------------------------------------------------------
// Test 1: Create task and query back by ID via get_task
// ---------------------------------------------------------------------------

#[test]
fn storage_create_and_query_task() {
    let db = test_db();

    let task = make_task(
        "task-001",
        "taobao",
        "12345",
        "iPhone Case",
        TaskStatus::Pending,
    );
    db.insert_task(&task).expect("insert_task should succeed");

    let fetched = db
        .get_task("task-001")
        .expect("get_task should not error")
        .expect("task should exist");

    assert_eq!(fetched.id, "task-001");
    assert_eq!(fetched.platform, "taobao");
    assert_eq!(fetched.item_id, "12345");
    assert_eq!(fetched.title, "iPhone Case");
    assert!(matches!(fetched.status, TaskStatus::Pending));
    assert_eq!(
        fetched.folder_path,
        Some("/tmp/egrab/taobao_12345_task-001/".to_string())
    );
}

// ---------------------------------------------------------------------------
// Test 2: Create multiple tasks and filter by platform, status, and keyword
// ---------------------------------------------------------------------------

#[test]
fn storage_task_history_with_filter() {
    let db = test_db();

    // Insert tasks across platforms and statuses.
    let t1 = make_task(
        "task-t1",
        "taobao",
        "111",
        "Apple iPhone 15",
        TaskStatus::Success,
    );
    let t2 = make_task(
        "task-t2",
        "jd",
        "222",
        "Samsung Galaxy S24",
        TaskStatus::Pending,
    );
    let t3 = make_task(
        "task-t3",
        "taobao",
        "333",
        "Xiaomi Phone Case",
        TaskStatus::Failed,
    );

    db.insert_task(&t1).unwrap();
    db.insert_task(&t2).unwrap();
    db.insert_task(&t3).unwrap();

    // Filter by platform = taobao.
    let filter_taobao = TaskFilter {
        platform: Some("taobao".to_string()),
        status: None,
        keyword: None,
        item_id: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
        offset: Some(0),
    };
    let results = db.query_tasks(&filter_taobao).unwrap();
    assert_eq!(results.len(), 2, "should find 2 taobao tasks");
    assert!(results.iter().all(|r| r.platform == "taobao"));

    // Filter by status = pending.
    let filter_pending = TaskFilter {
        platform: None,
        status: Some(TaskStatus::Pending),
        keyword: None,
        item_id: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
        offset: Some(0),
    };
    let results = db.query_tasks(&filter_pending).unwrap();
    assert_eq!(results.len(), 1, "should find 1 pending task");
    assert_eq!(results[0].id, "task-t2");

    // Filter by keyword = "iphone" (case-insensitive LIKE).
    let filter_keyword = TaskFilter {
        platform: None,
        status: None,
        keyword: Some("iphone".to_string()),
        item_id: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
        offset: Some(0),
    };
    let results = db.query_tasks(&filter_keyword).unwrap();
    assert_eq!(results.len(), 1, "should find 1 task matching 'iphone'");
    assert_eq!(results[0].title, "Apple iPhone 15");

    // No filters — should return all.
    let filter_all = TaskFilter {
        platform: None,
        status: None,
        keyword: None,
        item_id: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
        offset: Some(0),
    };
    let results = db.query_tasks(&filter_all).unwrap();
    assert_eq!(results.len(), 3, "should return all 3 tasks");
}

// ---------------------------------------------------------------------------
// Test 3: Duplicate detection — same platform + item_id should be detected,
//         but failed/cancelled tasks should be skipped.
// ---------------------------------------------------------------------------

#[test]
fn storage_duplicate_detection() {
    let db = test_db();

    let task = make_task(
        "task-dup",
        "taobao",
        "99999",
        "Dup Product",
        TaskStatus::Pending,
    );
    db.insert_task(&task).unwrap();

    // Should detect duplicate for pending task.
    let dup = db
        .check_duplicate("taobao", "99999")
        .expect("check_duplicate should not error");
    assert_eq!(
        dup,
        Some("task-dup".to_string()),
        "should detect existing pending task"
    );

    // Different item_id should not be detected.
    let no_dup = db.check_duplicate("taobao", "88888").unwrap();
    assert_eq!(no_dup, None, "should not detect non-existent item");

    // Failed task should be skipped by duplicate check.
    let failed_task = make_task(
        "task-fail",
        "jd",
        "77777",
        "Failed Product",
        TaskStatus::Failed,
    );
    db.insert_task(&failed_task).unwrap();
    let skip_failed = db.check_duplicate("jd", "77777").unwrap();
    assert_eq!(
        skip_failed, None,
        "duplicate check should skip failed tasks"
    );

    // Cancelled task should also be skipped.
    let cancelled_task = make_task(
        "task-cancel",
        "jd",
        "66666",
        "Cancelled Product",
        TaskStatus::Cancelled,
    );
    db.insert_task(&cancelled_task).unwrap();
    let skip_cancelled = db.check_duplicate("jd", "66666").unwrap();
    assert_eq!(
        skip_cancelled, None,
        "duplicate check should skip cancelled tasks"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Force overwrite — replace_task deletes old task + images and inserts new
// ---------------------------------------------------------------------------

#[test]
fn storage_force_overwrite() {
    let db = test_db();

    let old_task = make_task(
        "task-old",
        "taobao",
        "55555",
        "Old Title",
        TaskStatus::Success,
    );
    db.insert_task(&old_task).unwrap();

    // Attach an image to the old task.
    let img = ImageIndexInput {
        task_id: "task-old".to_string(),
        image_type: ImageType::Cover,
        original_url: "https://img.example.com/old.jpg".to_string(),
        local_path: Some("cover/old.jpg".to_string()),
        width: Some(800),
        height: Some(600),
        size_bytes: Some(12345),
    };
    db.insert_image(&img).unwrap();

    // Verify old image exists.
    let old_images = db.get_images_for_task("task-old").unwrap();
    assert_eq!(old_images.len(), 1);

    // Build replacement task.
    let new_task = make_task(
        "task-new",
        "taobao",
        "55555",
        "New Title",
        TaskStatus::Pending,
    );

    // Replace old task with new task.
    db.replace_task("task-old", &new_task)
        .expect("replace_task should succeed");

    // Old task should be gone.
    let gone = db.get_task("task-old").unwrap();
    assert!(gone.is_none(), "old task should be deleted");

    // Old images should be gone.
    let old_images_gone = db.get_images_for_task("task-old").unwrap();
    assert_eq!(old_images_gone.len(), 0, "old images should be deleted");

    // New task should exist.
    let fetched = db
        .get_task("task-new")
        .unwrap()
        .expect("new task should exist");
    assert_eq!(fetched.title, "New Title");
    assert!(matches!(fetched.status, TaskStatus::Pending));
}

// ---------------------------------------------------------------------------
// Test 5: Update task status through multiple state transitions
// ---------------------------------------------------------------------------

#[test]
fn storage_update_task_status() {
    let db = test_db();

    let task = make_task(
        "task-update",
        "taobao",
        "44444",
        "Update Me",
        TaskStatus::Pending,
    );
    db.insert_task(&task).unwrap();

    // Verify initial status.
    let fetched = db.get_task("task-update").unwrap().unwrap();
    assert!(matches!(fetched.status, TaskStatus::Pending));

    // Transition: Pending -> Running.
    let updates_running = TaskUpdate {
        status: Some(TaskStatus::Running),
        title: Some("Update Me — Running".to_string()),
        folder_path: None,
    };
    db.update_task("task-update", &updates_running)
        .expect("update to running should succeed");

    let fetched = db.get_task("task-update").unwrap().unwrap();
    assert!(matches!(fetched.status, TaskStatus::Running));
    assert_eq!(fetched.title, "Update Me — Running");

    // Transition: Running -> Success.
    let updates_success = TaskUpdate {
        status: Some(TaskStatus::Success),
        title: None,
        folder_path: None,
    };
    db.update_task("task-update", &updates_success)
        .expect("update to success should succeed");

    let fetched = db.get_task("task-update").unwrap().unwrap();
    assert!(matches!(fetched.status, TaskStatus::Success));

    // Transition: Success -> Failed.
    let updates_failed = TaskUpdate {
        status: Some(TaskStatus::Failed),
        title: Some("Update Me — Failed".to_string()),
        folder_path: None,
    };
    db.update_task("task-update", &updates_failed)
        .expect("update to failed should succeed");

    let fetched = db.get_task("task-update").unwrap().unwrap();
    assert!(matches!(fetched.status, TaskStatus::Failed));
    assert_eq!(fetched.title, "Update Me — Failed");
}

// ---------------------------------------------------------------------------
// Test 6: Image index CRUD — insert images and query by task_id
// ---------------------------------------------------------------------------

#[test]
fn storage_image_index_crud() {
    let db = test_db();

    let task = make_task("task-img", "jd", "33333", "Image Test", TaskStatus::Success);
    db.insert_task(&task).unwrap();

    // Insert multiple images of different types.
    let cover = ImageIndexInput {
        task_id: "task-img".to_string(),
        image_type: ImageType::Cover,
        original_url: "https://img.jd.com/cover.jpg".to_string(),
        local_path: Some("cover/cover_001.jpg".to_string()),
        width: Some(800),
        height: Some(800),
        size_bytes: Some(45000),
    };
    let gallery1 = ImageIndexInput {
        task_id: "task-img".to_string(),
        image_type: ImageType::Gallery,
        original_url: "https://img.jd.com/gallery1.jpg".to_string(),
        local_path: Some("gallery/main_001.jpg".to_string()),
        width: Some(1200),
        height: Some(1200),
        size_bytes: Some(89000),
    };
    let gallery2 = ImageIndexInput {
        task_id: "task-img".to_string(),
        image_type: ImageType::Gallery,
        original_url: "https://img.jd.com/gallery2.jpg".to_string(),
        local_path: Some("gallery/main_002.jpg".to_string()),
        width: Some(1200),
        height: Some(1200),
        size_bytes: Some(92000),
    };
    let detail = ImageIndexInput {
        task_id: "task-img".to_string(),
        image_type: ImageType::Detail,
        original_url: "https://img.jd.com/detail.jpg".to_string(),
        local_path: None, // simulate download failure
        width: None,
        height: None,
        size_bytes: None,
    };

    let cover_id = db.insert_image(&cover).unwrap();
    let g1_id = db.insert_image(&gallery1).unwrap();
    let g2_id = db.insert_image(&gallery2).unwrap();
    let detail_id = db.insert_image(&detail).unwrap();

    // Verify IDs are assigned sequentially.
    assert!(cover_id > 0);
    assert_eq!(g1_id, cover_id + 1);
    assert_eq!(g2_id, g1_id + 1);
    assert_eq!(detail_id, g2_id + 1);

    // Query all images for the task.
    let images = db
        .get_images_for_task("task-img")
        .expect("get_images_for_task should succeed");
    assert_eq!(images.len(), 4, "should return all 4 images");

    // Verify cover image fields.
    let cover_record: &ImageRecord = images
        .iter()
        .find(|i| matches!(i.image_type, ImageType::Cover))
        .expect("cover image should exist");
    assert_eq!(cover_record.task_id, "task-img");
    assert_eq!(cover_record.original_url, "https://img.jd.com/cover.jpg");
    assert_eq!(
        cover_record.local_path,
        Some("cover/cover_001.jpg".to_string())
    );
    assert_eq!(cover_record.width, Some(800));
    assert_eq!(cover_record.height, Some(800));
    assert_eq!(cover_record.size_bytes, Some(45000));

    // Verify gallery images.
    let gallery_records: Vec<&ImageRecord> = images
        .iter()
        .filter(|i| matches!(i.image_type, ImageType::Gallery))
        .collect();
    assert_eq!(gallery_records.len(), 2);

    // Verify detail image with null local_path (download failure case).
    let detail_record: &ImageRecord = images
        .iter()
        .find(|i| matches!(i.image_type, ImageType::Detail))
        .expect("detail image should exist");
    assert_eq!(detail_record.local_path, None);
    assert_eq!(detail_record.width, None);
    assert_eq!(detail_record.height, None);
    assert_eq!(detail_record.size_bytes, None);

    // Query images for a non-existent task should return empty.
    let empty = db.get_images_for_task("no-such-task").unwrap();
    assert_eq!(empty.len(), 0, "should return empty for unknown task");
}
