// EGrab - Scraper Engine: Core Implementation
// Orchestrates the full scrape pipeline:
//   CDP connect/navigate → Parser → Downloader → Storage → Events
//
// Derived from: src/protocols/scraper-engine.ts, ARCHITECTURE 4.1
// Constraints:
//   - MVP: max 1 concurrent scrape task (TASK_ALREADY_RUNNING on overlap)
//   - force=false: duplicate tasks return DUPLICATE_TASK
//   - force=true: delete old task data and re-scrape
//   - Events: scrape:progress, scrape:complete, scrape:error

use crate::cdp::CdpManager;
use crate::downloader::{DownloadImageInput, ImageDownloader};
use crate::models::{
    ConnectionState, ErrorCode, IpcError, ScrapeStep, TaskResult, TaskStatus, TaskUpdate,
};
use crate::parser::{self, PageHandle};
use crate::storage::StorageEngine;
use async_trait::async_trait;
use tauri::{Emitter, Manager};

/// The scraper engine coordinates the full scrape pipeline.
///
/// It holds a Tauri AppHandle to access managed state (CdpManager, StorageEngine)
/// and emit events to the frontend.
pub struct ScraperEngine {
    app_handle: tauri::AppHandle,
}

impl ScraperEngine {
    /// Creates a new ScraperEngine with the given Tauri AppHandle.
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Returns a reference to the AppHandle.
    /// Used by IPC commands to clone the handle for async operations.
    pub fn app_handle(&self) -> &tauri::AppHandle {
        &self.app_handle
    }

    /// Prepares a scrape task (synchronous phase).
    ///
    /// Phase 1 of the scrape pipeline:
    /// 1. Identify platform and extract item_id from URL
    /// 2. Create task in storage (with duplicate check)
    ///
    /// Returns the task_id immediately. The caller should then spawn
    /// `run_scrape` in a background tokio task to execute the async
    /// scrape pipeline (CDP navigation, parsing, downloading, saving).
    pub async fn prepare_task(&self, url: &str, force: Option<bool>) -> Result<String, IpcError> {
        // Step 1: Find the appropriate parser for this URL.
        let platform_parser = parser::find_parser(url).ok_or_else(|| IpcError {
            code: ErrorCode::UnsupportedPlatform,
            message: format!("No parser found for URL: {}", url),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?;

        let platform = platform_parser.platform_id().to_string();
        let item_id = platform_parser.extract_item_id(url).map_err(|e| IpcError {
            code: ErrorCode::ItemIdExtractFailed,
            message: format!("Failed to extract item ID from URL {}: {}", url, e),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?;

        // Step 2: Create task in storage.
        let storage = self.app_handle.state::<tokio::sync::Mutex<StorageEngine>>();
        let storage_guard = storage.lock().await;
        let task = storage_guard.create_task(url, &platform, &item_id, force)?;
        let task_id = task.id.clone();
        drop(storage_guard);

        Ok(task_id)
    }

    /// Runs the scrape pipeline in the background (async phase).
    ///
    /// Phases 2-6 of the scrape pipeline:
    /// 2. Update task status to Running
    /// 3. Navigate to URL via CDP
    /// 4. Parse page data via platform parser
    /// 5. Download images via ImageDownloader
    /// 6. Save results via StorageEngine
    /// 7. Emit scrape:complete event
    ///
    /// This method is designed to be spawned in a background tokio task.
    /// All errors are emitted as events rather than returned.
    pub async fn run_scrape(&self, task_id: String, url: String) {
        tracing::info!(task_id = %task_id, url = %url, "run_scrape started");

        // Re-derive the platform parser from the URL.
        let platform_parser = match parser::find_parser(&url) {
            Some(p) => p,
            None => {
                self.emit_error(
                    &task_id,
                    "No parser found for URL",
                    false,
                    ErrorCode::UnsupportedPlatform,
                    ScrapeStep::Connecting,
                );
                self.fail_task(&task_id).await;
                return;
            }
        };
        let platform = platform_parser.platform_id().to_string();

        let storage = self.app_handle.state::<tokio::sync::Mutex<StorageEngine>>();

        // Get folder_path from the task created in prepare_task.
        let folder_path = {
            let storage_guard = storage.lock().await;
            match storage_guard.get_task_detail(&task_id) {
                Ok(detail) => detail.task.folder_path,
                Err(e) => {
                    drop(storage_guard);
                    self.emit_error(
                        &task_id,
                        &e.message,
                        e.recoverable,
                        e.code,
                        e.step.unwrap_or(ScrapeStep::Connecting),
                    );
                    return;
                }
            }
        };

        // Update task status to Running.
        {
            let storage_guard = storage.lock().await;
            if let Err(e) = storage_guard.update_task(
                &task_id,
                TaskUpdate {
                    status: Some(TaskStatus::Running),
                    title: None,
                    folder_path: None,
                },
            ) {
                drop(storage_guard);
                self.emit_error(
                    &task_id,
                    &e.message,
                    e.recoverable,
                    e.code,
                    e.step.unwrap_or(ScrapeStep::Connecting),
                );
                return;
            }
        }

        // Emit progress: connecting (10%).
        self.emit_progress(&task_id, 10, ScrapeStep::Connecting, "Connecting to browser");

        // Step 3: Navigate to URL via CDP.
        let cdp = self.app_handle.state::<CdpManager>();

        // Step 2.5: Ensure CDP is connected before navigating.
        let cdp_status = cdp.status().await;
        if !matches!(cdp_status, ConnectionState::Connected { .. }) {
            self.emit_progress(
                &task_id,
                15,
                ScrapeStep::Connecting,
                "正在自动检测浏览器连接...",
            );
            match cdp.auto_connect().await {
                Ok(_info) => {
                    self.emit_progress(
                        &task_id,
                        20,
                        ScrapeStep::Connecting,
                        "浏览器已连接，正在导航...",
                    );
                }
                Err(e) => {
                    self.emit_error(
                        &task_id,
                        &e.message,
                        e.recoverable,
                        e.code,
                        ScrapeStep::Connecting,
                    );
                    self.fail_task(&task_id).await;
                    return;
                }
            }
        }

        tracing::info!(task_id = %task_id, url = %url, "Navigating to URL via CDP...");
        if let Err(e) = cdp.navigate(&url).await {
            tracing::error!(task_id = %task_id, error = ?e, "CDP navigation failed");
            // Update task status to Failed.
            if let Ok(guard) = self.app_handle
                .state::<tokio::sync::Mutex<StorageEngine>>()
                .try_lock()
            {
                let _ = guard.update_task(
                    &task_id,
                    TaskUpdate {
                        status: Some(TaskStatus::Failed),
                        title: None,
                        folder_path: None,
                    },
                );
            }
            self.emit_error(
                &task_id,
                &e.message,
                e.recoverable,
                e.code,
                e.step.unwrap_or(ScrapeStep::PageLoading),
            );
            return;
        }
        tracing::info!(task_id = %task_id, "CDP navigation succeeded");

        // Wait for JS rendering: poll every 300ms up to 6 seconds.
        // chromiumoxide's evaluate() defaults to awaitPromise:false, so Promise-based
        // JS does NOT actually wait. Must use Rust-side sleep + synchronous JS checks.
        let wait_check_js = r#"
            (function() {
                var hasTaobao = document.querySelector('.sku-name, .itemInfo-wrap .sku-name, #J_DetailMeta, .tb-title');
                var hasJD = document.querySelector('#spec-n1 img, #detail-main, #detail-top, ._scoped_1nhp8_1, .sku-title-name');
                var hasImages = document.querySelector('#spec-list img, .J-p-img img, #spec-n1 img');
                return !!(hasTaobao || hasJD || hasImages);
            })()
        "#;
        let max_wait_ms: u32 = 6000;
        let poll_interval_ms: u64 = 300;
        let mut waited: u32 = 0;
        loop {
            match cdp.evaluate(wait_check_js).await {
                Ok(v) if v.as_bool() == Some(true) => break,
                _ => {}
            }
            waited += poll_interval_ms as u32;
            if waited >= max_wait_ms {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(poll_interval_ms)).await;
        }

        // Scroll step by step to trigger lazy loading of detail images.
        // Uses synchronous JS + Rust-side tokio::time::sleep for timing —
        // chromiumoxide's evaluate() defaults to awaitPromise:false so Promise-based
        // scroll JS would return immediately without actually completing.
        //
        // JD detail containers use fixed height + overflow:hidden + transform:scale,
        // which prevents window.scrollTo from reaching the actual image elements.
        // Force-expand them before scrolling.
        let _ = cdp.evaluate(r#"
            (function() {
                var ids = ['detail-main','detail-top','related-layout-head','related-layout-footer'];
                for (var i = 0; i < ids.length; i++) {
                    var el = document.getElementById(ids[i]);
                    if (el) {
                        el.style.height = 'auto';
                        el.style.overflow = 'visible';
                        el.style.maxHeight = 'none';
                    }
                }
                var scoped = document.querySelector('._scoped_1nhp8_1');
                if (scoped) {
                    scoped.querySelectorAll('*').forEach(function(c) {
                        var s = c.style;
                        if (s.overflow === 'hidden') s.overflow = 'visible';
                        var h = s.height || s.maxHeight;
                        if (h && h !== 'auto') {
                            s.height = 'auto';
                            s.maxHeight = 'none';
                        }
                    });
                }
                // Set all images in detail containers to eager loading.
                var containers = document.querySelectorAll(
                    '#detail-main, #detail-top, #related-layout-head, ._scoped_1nhp8_1'
                );
                containers.forEach(function(ct) {
                    ct.querySelectorAll('img').forEach(function(img) {
                        img.loading = 'eager';
                        img.decoding = 'sync';
                    });
                });
                return 'expanded';
            })()
        "#).await;
        let height_result = cdp.evaluate("document.body.scrollHeight")
            .await
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(5000.0) as u32;

        let step: u32 = 500;
        let mut current: u32 = 0;
        let scroll_delay_ms: u64 = 300;

        while current < height_result {
            current = (current + step).min(height_result);
            let js = format!(
                "(function() {{ window.scrollTo(0, {}); return document.body.scrollHeight; }})()",
                current
            );
            let _ = cdp.evaluate(&js).await;
            tokio::time::sleep(std::time::Duration::from_millis(scroll_delay_ms)).await;
        }

        // Wait 1.5s for lazy-loaded images to appear after scrolling.
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Scroll back to top.
        let _ = cdp.evaluate("window.scrollTo(0, 0)").await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Emit progress: page loading (30%).
        self.emit_progress(&task_id, 30, ScrapeStep::PageLoading, "Page loaded");

        // Step 4: Parse page data.
        self.emit_progress(&task_id, 40, ScrapeStep::Parsing, "Parsing product data");

        // Create a CdpPageHandle that wraps the CdpManager reference.
        // Safety: CdpManager is managed by Tauri and lives for the entire app lifetime.
        let cdp_ptr: *const CdpManager = cdp.inner();
        let page_handle = CdpPageHandle { cdp: cdp_ptr };
        let parse_result = match platform_parser.parse(&page_handle).await {
            Ok(result) => result,
            Err(e) => {
                self.emit_error(
                    &task_id,
                    &format!("Parse failed: {}", e),
                    true,
                    ErrorCode::ParseFailed,
                    ScrapeStep::Parsing,
                );
                self.fail_task(&task_id).await;
                return;
            }
        };

        // Emit progress: parsing complete (60%).
        self.emit_progress(&task_id, 60, ScrapeStep::Parsing, "Parsing complete");

        // Step 5: Download images.
        self.emit_progress(&task_id, 70, ScrapeStep::Downloading, "Downloading images");

        let product = match parse_result.product {
            Some(p) => p,
            None => {
                // Parsing completely failed — save raw data and mark as partial.
                let storage_guard = storage.lock().await;

                let _ = storage_guard.save_raw(&task_id, &parse_result.raw_data, &parse_result.errors);

                if let Err(e) = storage_guard.update_task(
                    &task_id,
                    TaskUpdate {
                        status: Some(TaskStatus::Partial),
                        title: None,
                        folder_path: None,
                    },
                ) {
                drop(storage_guard);
                self.emit_error(
                    &task_id,
                    &e.message,
                    e.recoverable,
                    e.code,
                    e.step.unwrap_or(ScrapeStep::Downloading),
                );
                return;
                }

                let task_result = TaskResult {
                    task_id: task_id.clone(),
                    status: TaskStatus::Partial,
                    folder_path,
                    product: None,
                    image_total: 0,
                    image_success: 0,
                    image_failed: 0,
                    errors: parse_result.errors,
                };

                self.emit_complete(&task_id, &task_result);
                return;
            }
        };

        // Build download list from product data.
        let download_inputs = build_download_inputs(&product);
        let image_total = download_inputs.len() as u32;

        // Perform image download.
        let downloader = ImageDownloader::new();
        let download_result = if let Some(ref fp) = folder_path {
            let concurrency = 3u32; // Default; could read from config in the future.
            downloader
                .download_images(fp, &platform, &download_inputs, concurrency, 3)
                .await
        } else {
            // No folder path — cannot download.
            crate::downloader::DownloadBatchResult {
                total: image_total,
                success: 0,
                failed: image_total,
                results: vec![],
            }
        };

        // Emit progress: downloading complete (90%).
        self.emit_progress(&task_id, 90, ScrapeStep::Downloading, "Images downloaded");

        // Step 6: Save results via StorageEngine.
        self.emit_progress(&task_id, 95, ScrapeStep::Saving, "Saving results");

        let final_status;
        let mut errors;
        {
            let storage_guard = storage.lock().await;

            // Save meta.json.
            let _ = storage_guard.save_meta(&task_id, &product);

            // Save raw.json.
            let _ = storage_guard.save_raw(&task_id, &parse_result.raw_data, &parse_result.errors);

            // Index downloaded images.
            for result in &download_result.results {
                if let Some(ref local_path) = result.local_path {
                    let _ = storage_guard.index_image(crate::models::ImageIndexInput {
                        task_id: task_id.clone(),
                        image_type: result.image_type.clone(),
                        original_url: result.original_url.clone(),
                        local_path: Some(local_path.clone()),
                        width: result.width,
                        height: result.height,
                        size_bytes: result.size_bytes,
                    });
                }
            }

            // Determine final task status.
            final_status = if download_result.failed > 0 && download_result.success > 0 {
                TaskStatus::Partial
            } else if download_result.failed > 0 && download_result.success == 0 {
                TaskStatus::Failed
            } else {
                TaskStatus::Success
            };

            errors = parse_result.errors;
            for result in &download_result.results {
                if let Some(ref err) = result.error {
                    errors.push(err.clone());
                }
            }

            if let Err(e) = storage_guard.update_task(
                &task_id,
                TaskUpdate {
                    status: Some(final_status.clone()),
                    title: Some(product.title.clone()),
                    folder_path: None,
                },
            ) {
                drop(storage_guard);
                self.emit_error(
                    &task_id,
                    &e.message,
                    e.recoverable,
                    e.code,
                    e.step.unwrap_or(ScrapeStep::Saving),
                );
                return;
            }
        } // storage_guard dropped

        // Auto-resize images to proportioned/ subdirectory.
        // Resize failure does NOT cause the task to be marked Failed.
        if let Some(ref fp) = folder_path {
            self.emit_progress(&task_id, 97, ScrapeStep::Saving, "Resizing images");
            let fp_clone = fp.clone();
            match tokio::task::spawn_blocking(move || {
                crate::resize::resize_images_in_folder(&fp_clone, "proportioned")
            })
            .await
            {
                Ok(Ok(resize_result)) => {
                    tracing::info!(
                        "Auto-resize complete: total={} resized={} skipped={} failed={}",
                        resize_result.total,
                        resize_result.resized,
                        resize_result.skipped,
                        resize_result.failed,
                    );
                }
                Ok(Err(e)) => {
                    tracing::warn!("Auto-resize failed: {:?}", e);
                }
                Err(e) => {
                    tracing::warn!("Auto-resize spawn_blocking failed: {:?}", e);
                }
            }
        }

        // Emit final progress and complete event.
        self.emit_progress(&task_id, 100, ScrapeStep::Completed, "Scrape complete");

        let task_result = TaskResult {
            task_id: task_id.clone(),
            status: final_status,
            folder_path,
            product: Some(product),
            image_total: download_result.total,
            image_success: download_result.success,
            image_failed: download_result.failed,
            errors,
        };

        self.emit_complete(&task_id, &task_result);
    }

    /// Helper: marks a task as Failed in storage with best-effort semantics.
    async fn fail_task(&self, task_id: &str) {
        if let Ok(guard) = self.app_handle
            .state::<tokio::sync::Mutex<StorageEngine>>()
            .try_lock()
        {
            let _ = guard.update_task(
                task_id,
                TaskUpdate {
                    status: Some(TaskStatus::Failed),
                    title: None,
                    folder_path: None,
                },
            );
        }
    }

    /// Cancels a running scrape task.
    ///
    /// MVP: marks the task as cancelled in storage. Full cancellation
    /// of in-progress CDP operations will be implemented in a future phase.
    pub async fn cancel_scrape(&self, task_id: &str) -> Result<bool, IpcError> {
        let storage = self.app_handle.state::<tokio::sync::Mutex<StorageEngine>>();
        let storage_guard = storage.lock().await;

        // Get current task detail to check if it can be cancelled.
        let detail = storage_guard.get_task_detail(task_id)?;

        let can_cancel = matches!(detail.task.status, TaskStatus::Pending | TaskStatus::Running);
        if !can_cancel {
            return Err(IpcError {
                code: ErrorCode::TaskCancelled,
                message: format!(
                    "Task {} is in status {:?} and cannot be cancelled",
                    task_id, detail.task.status
                ),
                recoverable: false,
                step: None,
                details: None,
            });
        }

        storage_guard.update_task(
            task_id,
            TaskUpdate {
                status: Some(TaskStatus::Cancelled),
                title: None,
                folder_path: None,
            },
        )?;

        tracing::info!(task_id = %task_id, "Task cancelled");
        Ok(true)
    }

    // ---- Event Emission Helpers ----

    /// Emits a scrape:progress event.
    fn emit_progress(
        &self,
        task_id: &str,
        percent: u32,
        step: ScrapeStep,
        message: &str,
    ) {
        tracing::info!(
            task_id = %task_id,
            percent = percent,
            step = ?step,
            message = %message,
            "Emitting progress"
        );
        let payload = serde_json::json!({
            "task_id": task_id,
            "percent": percent,
            "step": step,
            "message": message,
        });
        match self.app_handle.emit("scrape:progress", &payload) {
            Ok(_) => tracing::info!(task_id = %task_id, "scrape:progress event emitted successfully"),
            Err(e) => tracing::error!(task_id = %task_id, error = %e, "Failed to emit scrape:progress"),
        }
    }

    /// Emits a scrape:complete event.
    fn emit_complete(&self, task_id: &str, result: &TaskResult) {
        tracing::info!(
            task_id = %task_id,
            status = ?result.status,
            image_total = result.image_total,
            image_success = result.image_success,
            image_failed = result.image_failed,
            "Emitting complete"
        );
        let payload = serde_json::json!({
            "task_id": task_id,
            "result": result,
        });
        let _ = self.app_handle.emit("scrape:complete", &payload);
    }

    /// Emits a scrape:error event.
    fn emit_error(
        &self,
        task_id: &str,
        error: &str,
        recoverable: bool,
        error_code: ErrorCode,
        step: ScrapeStep,
    ) {
        tracing::warn!(
            task_id = %task_id,
            error = %error,
            recoverable = recoverable,
            error_code = ?error_code,
            step = ?step,
            "Emitting error"
        );
        let payload = serde_json::json!({
            "task_id": task_id,
            "error": error,
            "recoverable": recoverable,
            "error_code": error_code,
            "step": step,
        });
        let _ = self.app_handle.emit("scrape:error", &payload);
    }
}

// ---- CDP PageHandle Implementation ----

/// CDP-backed PageHandle implementation.
///
/// Wraps a raw pointer to CdpManager to provide the PageHandle trait
/// interface required by platform parsers.
///
/// # Safety
/// The CdpManager is managed by Tauri's state system and lives for the
/// entire application lifetime. The CdpPageHandle only exists within the
/// scope of a start_scrape call, so the pointer remains valid.
struct CdpPageHandle {
    cdp: *const CdpManager,
}

// Safety: CdpManager is Send + Sync (uses tokio::sync::Mutex internally).
// The raw pointer is only used within the scope of start_scrape, where
// the CdpManager is guaranteed to be alive (managed by Tauri state).
unsafe impl Send for CdpPageHandle {}
unsafe impl Sync for CdpPageHandle {}

#[async_trait]
impl PageHandle for CdpPageHandle {
    async fn url(&self) -> anyhow::Result<String> {
        // Safety: the pointer is valid for the lifetime of start_scrape.
        let cdp = unsafe { &*self.cdp };
        let val = cdp.evaluate("document.URL").await
            .map_err(|e| anyhow::anyhow!("{}", e.message))?;
        Ok(val.as_str().unwrap_or("").to_string())
    }

    async fn title(&self) -> anyhow::Result<String> {
        let cdp = unsafe { &*self.cdp };
        let val = cdp.evaluate("document.title").await
            .map_err(|e| anyhow::anyhow!("{}", e.message))?;
        Ok(val.as_str().unwrap_or("").to_string())
    }

    async fn evaluate(&self, script: &str) -> anyhow::Result<serde_json::Value> {
        let cdp = unsafe { &*self.cdp };
        cdp.evaluate(script).await
            .map_err(|e| anyhow::anyhow!("{}", e.message))
    }

    async fn content(&self) -> anyhow::Result<String> {
        let cdp = unsafe { &*self.cdp };
        let val = cdp.evaluate("document.documentElement.outerHTML").await
            .map_err(|e| anyhow::anyhow!("{}", e.message))?;
        Ok(val.as_str().unwrap_or("").to_string())
    }
}

// ---- Helper Functions ----

/// Builds the list of image download inputs from a ProductData.
/// Filters out images with empty or whitespace-only original_url to avoid
/// reqwest 'relative URL without a base' errors.
fn build_download_inputs(product: &crate::models::ProductData) -> Vec<DownloadImageInput> {
    use crate::models::ImageType;

    let mut inputs = Vec::new();

    // Cover image.
    if !product.cover.original_url.trim().is_empty() {
        inputs.push(DownloadImageInput {
            image_type: ImageType::Cover,
            image: product.cover.clone(),
            relative_path: "cover/cover_001.jpg".to_string(),
        });
    } else {
        tracing::warn!("Cover image original_url is empty; skipping cover download");
    }

    // Gallery images.
    for (i, img) in product.gallery.iter().enumerate() {
        if img.original_url.trim().is_empty() {
            tracing::warn!(index = i, "Gallery image original_url is empty; skipping");
            continue;
        }
        let ext = extract_extension(&img.original_url).unwrap_or("jpg");
        inputs.push(DownloadImageInput {
            image_type: ImageType::Gallery,
            image: img.clone(),
            relative_path: format!("gallery/main_{:03}.{}", i + 1, ext),
        });
    }

    // Detail images.
    for (i, img) in product.detail_images.iter().enumerate() {
        if img.original_url.trim().is_empty() {
            tracing::warn!(index = i, "Detail image original_url is empty; skipping");
            continue;
        }
        let ext = extract_extension(&img.original_url).unwrap_or("jpg");
        inputs.push(DownloadImageInput {
            image_type: ImageType::Detail,
            image: img.clone(),
            relative_path: format!("detail/detail_{:03}.{}", i + 1, ext),
        });
    }

    // SKU images.
    for (name, img) in &product.sku_images {
        if img.original_url.trim().is_empty() {
            tracing::warn!(sku = %name, "SKU image original_url is empty; skipping");
            continue;
        }
        let ext = extract_extension(&img.original_url).unwrap_or("jpg");
        let safe_name = sanitize_filename(name);
        inputs.push(DownloadImageInput {
            image_type: ImageType::Sku,
            image: img.clone(),
            relative_path: format!("sku/sku_{}.{}", safe_name, ext),
        });
    }

    inputs
}

/// Extracts the file extension from a URL path.
fn extract_extension(url: &str) -> Option<&str> {
    let path = url.split('?').next()?.split('#').next()?;
    let filename = path.rsplit('/').next()?;
    let dot_pos = filename.rfind('.')?;
    let ext = &filename[dot_pos + 1..];
    match ext {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => Some(ext),
        _ => None,
    }
}

/// Sanitizes a string for use as a filename component.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_extension_jpg() {
        assert_eq!(extract_extension("https://img.example.com/photo.jpg"), Some("jpg"));
    }

    #[test]
    fn test_extract_extension_png_with_query() {
        assert_eq!(
            extract_extension("https://img.example.com/photo.png?width=800"),
            Some("png")
        );
    }

    #[test]
    fn test_extract_extension_no_ext() {
        assert_eq!(extract_extension("https://img.example.com/photo"), None);
    }

    #[test]
    fn test_extract_extension_unrecognized() {
        assert_eq!(extract_extension("https://img.example.com/photo.txt"), None);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("color-red"), "color_red");
        assert_eq!(sanitize_filename("Blue"), "Blue");
    }

    #[test]
    fn test_build_download_inputs() {
        use crate::models::{Description, ImageRef, PriceRange, ProductData, ShopInfo};
        use std::collections::HashMap;

        let product = ProductData {
            title: "Test".to_string(),
            cover: ImageRef {
                original_url: "https://img.example.com/cover.jpg".to_string(),
                thumbnail_url: "".to_string(),
                local_path: None,
            },
            gallery: vec![
                ImageRef {
                    original_url: "https://img.example.com/main1.jpg".to_string(),
                    thumbnail_url: "".to_string(),
                    local_path: None,
                },
            ],
            description: Description {
                text: "".to_string(),
                html: None,
                specs: vec![],
            },
            detail_images: vec![],
            skus: vec![],
            sku_images: HashMap::new(),
            price: PriceRange {
                min_price: 0.0,
                max_price: 0.0,
                currency: "CNY".to_string(),
            },
            shop: ShopInfo {
                name: "".to_string(),
                url: "".to_string(),
            },
        };

        let inputs = build_download_inputs(&product);
        assert_eq!(inputs.len(), 2); // 1 cover + 1 gallery
        assert_eq!(inputs[0].relative_path, "cover/cover_001.jpg");
        assert_eq!(inputs[1].relative_path, "gallery/main_001.jpg");
    }

    #[test]
    fn test_build_download_inputs_skips_empty_urls() {
        use crate::models::{Description, ImageRef, PriceRange, ProductData, ShopInfo};
        use std::collections::HashMap;

        let mut sku_images = HashMap::new();
        sku_images.insert(
            "Red".to_string(),
            ImageRef {
                original_url: "".to_string(),
                thumbnail_url: "".to_string(),
                local_path: None,
            },
        );

        let product = ProductData {
            title: "Test".to_string(),
            cover: ImageRef {
                original_url: "".to_string(),
                thumbnail_url: "".to_string(),
                local_path: None,
            },
            gallery: vec![
                ImageRef {
                    original_url: "".to_string(),
                    thumbnail_url: "".to_string(),
                    local_path: None,
                },
                ImageRef {
                    original_url: "https://img.example.com/main2.jpg".to_string(),
                    thumbnail_url: "".to_string(),
                    local_path: None,
                },
            ],
            description: Description {
                text: "".to_string(),
                html: None,
                specs: vec![],
            },
            detail_images: vec![
                ImageRef {
                    original_url: "".to_string(),
                    thumbnail_url: "".to_string(),
                    local_path: None,
                },
            ],
            skus: vec![],
            sku_images,
            price: PriceRange {
                min_price: 0.0,
                max_price: 0.0,
                currency: "CNY".to_string(),
            },
            shop: ShopInfo {
                name: "".to_string(),
                url: "".to_string(),
            },
        };

        let inputs = build_download_inputs(&product);
        assert_eq!(inputs.len(), 1); // only main2.jpg survives
        assert_eq!(inputs[0].relative_path, "gallery/main_002.jpg");
    }
}
