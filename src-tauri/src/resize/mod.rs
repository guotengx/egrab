// EGrab - Image Resize Module
// Detects and resizes oversized product images with proportional scaling.

use crate::models::{ErrorCode, IpcError, ScrapeStep};
use std::path::Path;

/// Result of a resize operation on a single image.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResizeResult {
    /// Total images scanned
    pub total: u32,
    /// Images that were resized
    pub resized: u32,
    /// Images that were skipped (already within limits)
    pub skipped: u32,
    /// Images that failed to process
    pub failed: u32,
    /// Details for each image
    pub details: Vec<ResizeDetail>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResizeDetail {
    pub path: String,
    pub original_width: u32,
    pub original_height: u32,
    pub new_width: Option<u32>,
    pub new_height: Option<u32>,
    pub action: String, // "resized", "skipped", "failed"
    pub error: Option<String>,
}

const MAX_WIDTH: u32 = 1080; // 1200 * 0.9
const MAX_HEIGHT: u32 = 1350; // 1500 * 0.9

/// Resize all images in a folder (cover/, gallery/, detail/, sku/ subdirs).
pub fn resize_images_in_folder(folder_path: &str) -> Result<ResizeResult, IpcError> {
    let base = Path::new(folder_path);
    if !base.exists() || !base.is_dir() {
        return Err(IpcError {
            code: ErrorCode::UnknownError,
            message: format!("Folder not found: {}", folder_path),
            recoverable: true,
            step: Some(ScrapeStep::Saving),
            details: None,
        });
    }

    let mut result = ResizeResult {
        total: 0,
        resized: 0,
        skipped: 0,
        failed: 0,
        details: Vec::new(),
    };

    let subdirs = ["cover", "gallery", "detail", "sku"];
    for sub in &subdirs {
        let sub_path = base.join(sub);
        if !sub_path.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&sub_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                // Only process image files
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "avif" | "bmp"
                ) {
                    continue;
                }

                result.total += 1;

                match process_single_image(&path) {
                    Ok(detail) => {
                        if detail.action == "resized" {
                            result.resized += 1;
                        } else {
                            result.skipped += 1;
                        }
                        result.details.push(detail);
                    }
                    Err(e) => {
                        result.failed += 1;
                        result.details.push(ResizeDetail {
                            path: path.to_string_lossy().to_string(),
                            original_width: 0,
                            original_height: 0,
                            new_width: None,
                            new_height: None,
                            action: "failed".to_string(),
                            error: Some(e),
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

fn process_single_image(path: &Path) -> Result<ResizeDetail, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open: {}", e))?;
    let (w, h) = (img.width(), img.height());

    let scale = f64::min(
        MAX_WIDTH as f64 / w as f64,
        f64::min(MAX_HEIGHT as f64 / h as f64, 1.0),
    );

    if scale >= 1.0 {
        return Ok(ResizeDetail {
            path: path.to_string_lossy().to_string(),
            original_width: w,
            original_height: h,
            new_width: None,
            new_height: None,
            action: "skipped".to_string(),
            error: None,
        });
    }

    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;

    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);
    resized
        .save(path)
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(ResizeDetail {
        path: path.to_string_lossy().to_string(),
        original_width: w,
        original_height: h,
        new_width: Some(new_w),
        new_height: Some(new_h),
        action: "resized".to_string(),
        error: None,
    })
}
