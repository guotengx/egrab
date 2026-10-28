// EGrab - Image Resize Module
// Detects and resizes oversized product images with proportional scaling.

use crate::models::{ErrorCode, IpcError, ResizeDetail, ResizeResult, ScrapeStep};
use std::path::{Path, PathBuf};

const MAX_WIDTH: u32 = 1080; // 1200 * 0.9
const MAX_HEIGHT: u32 = 1350; // 1500 * 0.9

/// Resize all images in a folder (cover/, gallery/, detail/, sku/ subdirs).
/// Resized images are written to `{folder_path}/{output_base_name}/` preserving the
/// original subdirectory structure. Originals are never modified.
pub fn resize_images_in_folder(
    folder_path: &str,
    output_base_name: &str,
) -> Result<ResizeResult, IpcError> {
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

    let output_base = base.join(output_base_name);
    std::fs::create_dir_all(&output_base).map_err(|e| IpcError {
        code: ErrorCode::UnknownError,
        message: format!("Failed to create output directory: {}", e),
        recoverable: true,
        step: Some(ScrapeStep::Saving),
        details: None,
    })?;

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

                match process_single_image(&path, base, &output_base) {
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

fn process_single_image(
    path: &Path,
    base_dir: &Path,
    output_base: &Path,
) -> Result<ResizeDetail, String> {
    // Detect the format from the file content, not the extension: archives
    // from v0.2.1 and earlier contain PNG data saved as `.jpg`, which the
    // extension-based `image::open` cannot decode.
    let decoded = image::ImageReader::open(path)
        .and_then(|reader| reader.with_guessed_format())
        .map_err(|e| e.to_string())
        .and_then(|reader| reader.decode().map_err(|e| e.to_string()));
    let img = match decoded {
        Ok(img) => img,
        // Never drop an image from the output folder: keep the original instead.
        Err(e) => {
            return copy_original(
                path,
                base_dir,
                output_base,
                format!("decode failed, original copied: {}", e),
            )
        }
    };
    let (w, h) = (img.width(), img.height());

    let scale = f64::min(
        MAX_WIDTH as f64 / w as f64,
        f64::min(MAX_HEIGHT as f64 / h as f64, 1.0),
    );

    if scale >= 1.0 {
        let output_path = output_path_for(path, base_dir, output_base)?;
        std::fs::copy(path, &output_path).map_err(|e| format!("Failed to copy: {}", e))?;

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

    let output_path = output_path_for(path, base_dir, output_base)?;
    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // JPEG has no alpha channel: drop it before encoding to a .jpg/.jpeg target,
    // otherwise saving an RGBA image (e.g. a PNG) under a .jpg name fails.
    let is_jpeg_target = output_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        .unwrap_or(false);
    let saved = if is_jpeg_target {
        image::DynamicImage::ImageRgb8(resized.to_rgb8()).save(&output_path)
    } else {
        resized.save(&output_path)
    };
    if let Err(e) = saved {
        return copy_original(
            path,
            base_dir,
            output_base,
            format!("encode failed, original copied: {}", e),
        );
    }

    Ok(ResizeDetail {
        path: output_path.to_string_lossy().to_string(),
        original_width: w,
        original_height: h,
        new_width: Some(new_w),
        new_height: Some(new_h),
        action: "resized".to_string(),
        error: None,
    })
}

/// Output path under `output_base` that mirrors the path relative to `base_dir`,
/// with parent directories created.
fn output_path_for(path: &Path, base_dir: &Path, output_base: &Path) -> Result<PathBuf, String> {
    let relative = path
        .strip_prefix(base_dir)
        .map_err(|e| format!("strip prefix: {}", e))?;
    let output_path = output_base.join(relative);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {}", e))?;
    }
    Ok(output_path)
}

/// Copies the original image unchanged when it cannot be decoded or re-encoded,
/// so the output folder stays complete. Counted as "skipped" with the reason
/// recorded in `error`.
fn copy_original(
    path: &Path,
    base_dir: &Path,
    output_base: &Path,
    reason: String,
) -> Result<ResizeDetail, String> {
    let output_path = output_path_for(path, base_dir, output_base)?;
    std::fs::copy(path, &output_path).map_err(|e| format!("Failed to copy: {}", e))?;
    tracing::warn!(path = %path.display(), reason = %reason, "Image kept as original");

    Ok(ResizeDetail {
        path: path.to_string_lossy().to_string(),
        original_width: 0,
        original_height: 0,
        new_width: None,
        new_height: None,
        action: "skipped".to_string(),
        error: Some(reason),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_task_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("egrab-resize-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cover")).expect("create temp cover dir");
        dir
    }

    #[test]
    fn png_data_saved_as_jpg_is_resized_not_dropped() {
        // v0.2.1 saved JD PNG covers as cover_001.jpg; they were silently missing
        // from proportioned/ because decoding went by extension.
        let dir = temp_task_dir("png-as-jpg");
        let img = image::RgbaImage::from_pixel(2000, 10, image::Rgba([10, 20, 30, 128]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode test png");
        std::fs::write(dir.join("cover").join("cover_001.jpg"), png.into_inner())
            .expect("write test image");

        let result = resize_images_in_folder(dir.to_str().expect("utf-8 temp path"), "proportioned")
            .expect("resize folder");
        assert_eq!(result.failed, 0);
        assert_eq!(result.resized, 1);
        assert!(dir.join("proportioned").join("cover").join("cover_001.jpg").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn undecodable_image_is_copied_as_is() {
        let dir = temp_task_dir("garbage");
        std::fs::write(dir.join("cover").join("cover_001.jpg"), b"not an image")
            .expect("write test file");

        let result = resize_images_in_folder(dir.to_str().expect("utf-8 temp path"), "proportioned")
            .expect("resize folder");
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 1);
        assert!(dir.join("proportioned").join("cover").join("cover_001.jpg").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
