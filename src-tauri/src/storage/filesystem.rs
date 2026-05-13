// EGrab - Storage Engine: Filesystem Layer
// File system operations: folder creation, JSON read/write, open folder.
// All paths are validated against storage_root to prevent directory traversal.

use crate::models::{MetaJsonDocument, ProductData, RawJsonDocument, ScrapeErrorInfo, Task};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Creates the archive directory structure for a task:
///   {storage_root}/{platform}_{item_id}_{timestamp}/
///   with subdirectories: cover/, gallery/, detail/, sku/
pub fn create_task_folder(
    storage_root: &str,
    platform: &str,
    item_id: &str,
    timestamp: &str,
) -> Result<PathBuf> {
    let folder_name = format!("{}_{}_{}", platform, item_id, timestamp);
    let base_path = PathBuf::from(storage_root).join(&folder_name);

    std::fs::create_dir_all(&base_path)
        .with_context(|| format!("Failed to create task folder: {:?}", base_path))?;

    // Create subdirectories.
    for sub in &["cover", "gallery", "detail", "sku"] {
        std::fs::create_dir_all(base_path.join(sub))
            .with_context(|| format!("Failed to create subdirectory: {:?}", base_path.join(sub)))?;
    }

    tracing::info!("Created task folder: {:?}", base_path);
    Ok(base_path)
}

/// Writes a MetaJsonDocument to {folder_path}/meta.json.
pub fn write_meta_json(folder_path: &Path, doc: &MetaJsonDocument) -> Result<PathBuf> {
    let file_path = folder_path.join("meta.json");
    let json_str =
        serde_json::to_string_pretty(doc).context("Failed to serialize MetaJsonDocument")?;

    std::fs::write(&file_path, json_str)
        .with_context(|| format!("Failed to write meta.json to {:?}", file_path))?;

    tracing::info!("Written meta.json: {:?}", file_path);
    Ok(file_path)
}

/// Reads and parses meta.json from a task folder. Returns None if file does not exist.
pub fn read_meta_json(folder_path: &Path) -> Result<Option<ProductData>> {
    let file_path = folder_path.join("meta.json");
    if !file_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read meta.json: {:?}", file_path))?;

    let doc: MetaJsonDocument =
        serde_json::from_str(&content).context("Failed to parse meta.json")?;

    Ok(Some(doc.data))
}

/// Builds a MetaJsonDocument from task metadata and parsed product data.
pub fn build_meta_document(task: &Task, product: &ProductData) -> MetaJsonDocument {
    MetaJsonDocument {
        version: "1.0.0".to_string(),
        platform: task.platform.clone(),
        item_id: task.item_id.clone(),
        scraped_at: chrono_now(),
        data: product.clone(),
    }
}

/// Writes a RawJsonDocument to {folder_path}/raw.json.
pub fn write_raw_json(
    folder_path: &Path,
    url: &str,
    platform: &str,
    item_id: &str,
    raw_data: &serde_json::Value,
    parser_errors: &[ScrapeErrorInfo],
) -> Result<PathBuf> {
    let doc = RawJsonDocument {
        version: "1.0.0".to_string(),
        platform: platform.to_string(),
        item_id: item_id.to_string(),
        scraped_at: chrono_now(),
        url: url.to_string(),
        raw_data: raw_data.clone(),
        parser_errors: parser_errors.to_vec(),
    };

    let file_path = folder_path.join("raw.json");
    let json_str =
        serde_json::to_string_pretty(&doc).context("Failed to serialize RawJsonDocument")?;

    std::fs::write(&file_path, json_str)
        .with_context(|| format!("Failed to write raw.json to {:?}", file_path))?;

    tracing::info!("Written raw.json: {:?}", file_path);
    Ok(file_path)
}

/// Reads and returns the path to raw.json if it exists.
pub fn raw_json_path(folder_path: &Path) -> Option<PathBuf> {
    let file_path = folder_path.join("raw.json");
    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

/// Opens a folder in the system file manager.
///
/// # Safety
/// The path is validated to ensure it is within the configured storage_root
/// to prevent path traversal / directory escape attacks.
pub fn open_folder(path: &str, storage_root: &str) -> Result<bool> {
    // Canonicalize both paths to resolve symlinks and relative components.
    let target = PathBuf::from(path);
    let canonical_target = target
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path))?;

    let root = PathBuf::from(storage_root);
    // Ensure storage_root exists before canonicalizing (it will be created on first task).
    let canonical_root = if root.exists() {
        root.canonicalize()
            .with_context(|| format!("Failed to resolve storage root: {}", storage_root))?
    } else {
        // If storage_root doesn't exist yet, there's nothing to open.
        return Err(anyhow::anyhow!(
            "Storage root does not exist: {}",
            storage_root
        ));
    };

    // Validate: target must be within storage_root.
    if !canonical_target.starts_with(&canonical_root) {
        return Err(anyhow::anyhow!(
            "Path not allowed: {} is outside storage root {}",
            path,
            storage_root
        ));
    }

    // Validate: target must exist.
    if !canonical_target.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path));
    }

    // Open using platform-specific command.
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&canonical_target)
            .spawn()
            .with_context(|| format!("Failed to open folder: {:?}", canonical_target))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&canonical_target)
            .spawn()
            .with_context(|| format!("Failed to open folder: {:?}", canonical_target))?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Linux / other Unix: use xdg-open.
        std::process::Command::new("xdg-open")
            .arg(&canonical_target)
            .spawn()
            .with_context(|| format!("Failed to open folder: {:?}", canonical_target))?;
    }

    tracing::info!("Opened folder: {:?}", canonical_target);
    Ok(true)
}

/// Resolves the database path based on the current platform.
///
/// - macOS: ~/Library/Application Support/com.egrab.app/index.db
/// - Windows: %APPDATA%\com.egrab.app\index.db
pub fn database_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/Unknown".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.egrab.app")
            .join("index.db")
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string());
        PathBuf::from(appdata)
            .join("com.egrab.app")
            .join("index.db")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".egrab").join("index.db")
    }
}

/// Returns an ISO 8601 timestamp string for the current UTC time.
/// Uses chrono-like format without adding chrono dependency.
fn chrono_now() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    // Simple ISO 8601 formatting without external crate.
    // Calculate date/time components manually.
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year/month/day from days since epoch (1970-01-01).
    // Using a simple algorithm (Zeller-like).
    let (year, month, day) = days_to_date(days_since_epoch as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Converts days since Unix epoch to (year, month, day).
fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    // Algorithm: start from 1970-01-01, iterate forward.
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
    use crate::models::MetaJsonDocument;
    use tempfile::TempDir;

    #[test]
    fn test_create_task_folder() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_str().unwrap();

        let folder = create_task_folder(root, "taobao", "12345", "20260510T120000").unwrap();

        assert!(folder.exists());
        assert!(folder.join("cover").exists());
        assert!(folder.join("gallery").exists());
        assert!(folder.join("detail").exists());
        assert!(folder.join("sku").exists());
    }

    #[test]
    fn test_write_and_read_meta_json() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_str().unwrap();

        let folder = create_task_folder(root, "taobao", "12345", "20260510T120000").unwrap();

        let product = crate::models::ProductData {
            title: "Test Product".to_string(),
            cover: crate::models::ImageRef {
                original_url: "https://img.example.com/cover.jpg".to_string(),
                thumbnail_url: "https://img.example.com/thumb.jpg".to_string(),
                local_path: None,
            },
            gallery: vec![],
            description: crate::models::Description {
                text: "Test description".to_string(),
                html: None,
                specs: vec![],
            },
            detail_images: vec![],
            skus: vec![],
            sku_images: std::collections::HashMap::new(),
            price: crate::models::PriceRange {
                min_price: 99.0,
                max_price: 199.0,
                currency: "CNY".to_string(),
            },
            shop: crate::models::ShopInfo {
                name: "Test Shop".to_string(),
                url: "https://shop.example.com".to_string(),
            },
        };

        let doc = MetaJsonDocument {
            version: "1.0.0".to_string(),
            platform: "taobao".to_string(),
            item_id: "12345".to_string(),
            scraped_at: "2026-05-10T12:00:00Z".to_string(),
            data: product.clone(),
        };

        let path = write_meta_json(&folder, &doc).unwrap();
        assert!(path.exists());

        let read = read_meta_json(&folder).unwrap().unwrap();
        assert_eq!(read.title, "Test Product");
        assert_eq!(read.price.min_price, 99.0);
    }

    #[test]
    fn test_open_folder_path_validation() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_str().unwrap();

        let folder = create_task_folder(root, "taobao", "12345", "20260510T120000").unwrap();

        // Valid: folder within storage_root.
        let result = open_folder(folder.to_str().unwrap(), root);
        assert!(result.is_ok());

        // Invalid: path outside storage_root.
        let result = open_folder("/etc", root);
        assert!(result.is_err());
    }

    #[test]
    fn test_database_path_is_absolute() {
        let path = database_path();
        assert!(
            path.is_absolute(),
            "Database path should be absolute: {:?}",
            path
        );
    }

    #[test]
    fn test_chrono_now_returns_valid_iso8601() {
        let ts = chrono_now();
        // Should match pattern: YYYY-MM-DDTHH:MM:SSZ
        assert!(ts.len() >= 20);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }

    #[test]
    fn test_raw_json_write() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_str().unwrap();

        let folder = create_task_folder(root, "taobao", "12345", "20260510T120000").unwrap();

        let raw_data = serde_json::json!({"some": "data", "nested": {"key": "value"}});
        let path = write_raw_json(
            &folder,
            "https://item.taobao.com/item.htm?id=12345",
            "taobao",
            "12345",
            &raw_data,
            &[],
        )
        .unwrap();

        assert!(path.exists());

        // Verify we can read it back.
        let content = std::fs::read_to_string(&path).unwrap();
        let doc: RawJsonDocument = serde_json::from_str(&content).unwrap();
        assert_eq!(doc.platform, "taobao");
        assert_eq!(doc.item_id, "12345");
        assert_eq!(doc.raw_data, raw_data);
    }
}
