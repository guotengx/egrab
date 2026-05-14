// EGrab - Resize Module Integration Test
// Verifies: originals untouched, proportioned/ output correct, skip logic, result counts

use std::fs;
use std::path::Path;

// Import the resize module from the egrab crate
use egrab::resize;

const MAX_WIDTH: u32 = 1080;
const MAX_HEIGHT: u32 = 1350;

/// Helper: create a solid-color test image and save it to `dest`.
fn create_test_image(dest: &Path, width: u32, height: u32) -> anyhow::Result<()> {
    let mut img = image::RgbImage::new(width, height);
    // Fill with a solid color (R=100, G=150, B=200)
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([100, 150, 200]);
    }
    img.save(dest)?;
    Ok(())
}

/// Build a temporary task-folder structure:
/// tmp_root/
///   cover/   test_large.jpg (2000x1500)  test_small.jpg (400x300)
///   gallery/ test_large.jpg (2000x1500)
///   detail/  (empty)
///   sku/     (empty)
fn build_test_structure(tmp_root: &Path) -> anyhow::Result<()> {
    for sub in ["cover", "gallery", "detail", "sku"] {
        fs::create_dir_all(tmp_root.join(sub))?;
    }
    // Large image: exceeds both MAX_WIDTH and MAX_HEIGHT → must be resized
    create_test_image(&tmp_root.join("cover/test_large.jpg"), 2000, 1500)?;
    // Small image: within limits → must be skipped
    create_test_image(&tmp_root.join("cover/test_small.jpg"), 400, 300)?;
    // Gallery large image: exceeds limits → must be resized
    create_test_image(&tmp_root.join("gallery/test_large.jpg"), 2000, 1500)?;
    Ok(())
}

#[test]
fn resize_preserves_originals_and_outputs_proportioned() -> anyhow::Result<()> {
    // ── 1. Build temp structure ──────────────────────────────────────────────
    let tmp_dir = tempfile::tempdir()?;
    let task_dir = tmp_dir.path().join("task_001");
    build_test_structure(&task_dir)?;

    // Record original file sizes / dimensions before resize
    let orig_cover_large = task_dir.join("cover/test_large.jpg");
    let orig_cover_small = task_dir.join("cover/test_small.jpg");
    let orig_gallery_large = task_dir.join("gallery/test_large.jpg");

    let orig_cover_large_info = image::open(&orig_cover_large)?.dimensions();
    let orig_cover_small_info = image::open(&orig_cover_small)?.dimensions();
    let orig_gallery_large_info = image::open(&orig_gallery_large)?.dimensions();

    // ── 2. Call resize ───────────────────────────────────────────────────────
    let result = resize::resize_images_in_folder(task_dir.to_str().unwrap(), "proportioned")?;

    // ── 3. Assert ResizeResult counts ────────────────────────────────────────
    // 3 images total (2 cover + 1 gallery); detail/ and sku/ are empty
    assert_eq!(
        result.total, 3,
        "expected 3 total images, got {}",
        result.total
    );
    assert_eq!(
        result.resized, 2,
        "expected 2 resized images, got {}",
        result.resized
    );
    assert_eq!(
        result.skipped, 1,
        "expected 1 skipped image, got {}",
        result.skipped
    );
    assert_eq!(
        result.failed, 0,
        "expected 0 failed images, got {}",
        result.failed
    );
    assert_eq!(result.details.len(), 3, "expected 3 detail entries");

    // ── 4. Assert originals are untouched ────────────────────────────────────
    assert!(
        orig_cover_large.exists(),
        "original cover/test_large.jpg should still exist"
    );
    assert!(
        orig_cover_small.exists(),
        "original cover/test_small.jpg should still exist"
    );
    assert!(
        orig_gallery_large.exists(),
        "original gallery/test_large.jpg should still exist"
    );

    // Original dimensions must be unchanged
    assert_eq!(
        image::open(&orig_cover_large)?.dimensions(),
        orig_cover_large_info,
        "original cover_large dimensions changed"
    );
    assert_eq!(
        image::open(&orig_cover_small)?.dimensions(),
        orig_cover_small_info,
        "original cover_small dimensions changed"
    );
    assert_eq!(
        image::open(&orig_gallery_large)?.dimensions(),
        orig_gallery_large_info,
        "original gallery_large dimensions changed"
    );

    // ── 5. Assert proportioned/ output structure ─────────────────────────────
    let prop_dir = task_dir.join("proportioned");
    assert!(prop_dir.exists(), "proportioned/ directory should exist");

    // Subdirectory structure must mirror source
    for sub in ["cover", "gallery", "detail", "sku"] {
        assert!(
            prop_dir.join(sub).exists(),
            "proportioned/{sub}/ should exist"
        );
    }

    // ── 6. Assert resized images ─────────────────────────────────────────────
    let prop_cover_large = prop_dir.join("cover/test_large.jpg");
    assert!(
        prop_cover_large.exists(),
        "proportioned/cover/test_large.jpg should exist"
    );
    let (w, h) = image::open(&prop_cover_large)?.dimensions();
    assert!(
        w <= MAX_WIDTH && h <= MAX_HEIGHT,
        "proportioned/cover/test_large.jpg {}x{} exceeds MAX {}x{}",
        w,
        h,
        MAX_WIDTH,
        MAX_HEIGHT
    );

    let prop_gallery_large = prop_dir.join("gallery/test_large.jpg");
    assert!(
        prop_gallery_large.exists(),
        "proportioned/gallery/test_large.jpg should exist"
    );
    let (w2, h2) = image::open(&prop_gallery_large)?.dimensions();
    assert!(
        w2 <= MAX_WIDTH && h2 <= MAX_HEIGHT,
        "proportioned/gallery/test_large.jpg {}x{} exceeds MAX {}x{}",
        w2,
        h2,
        MAX_WIDTH,
        MAX_HEIGHT
    );

    // ── 7. Assert skipped image is NOT in proportioned/ ──────────────────────
    let prop_cover_small = prop_dir.join("cover/test_small.jpg");
    assert!(
        !prop_cover_small.exists(),
        "proportioned/cover/test_small.jpg should NOT exist (image was skipped)"
    );

    // ── 8. Assert ResizeDetail entries ───────────────────────────────────────
    let detail_cover_large = result
        .details
        .iter()
        .find(|d| d.path.contains("cover/test_large.jpg"))
        .expect("detail for cover/test_large.jpg missing");
    assert_eq!(detail_cover_large.action, "resized");
    assert_eq!(
        detail_cover_large.original_width, 2000,
        "original_width mismatch for cover_large"
    );
    assert_eq!(
        detail_cover_large.original_height, 1500,
        "original_height mismatch for cover_large"
    );
    assert!(detail_cover_large.new_width.is_some());
    assert!(detail_cover_large.new_height.is_some());

    let detail_cover_small = result
        .details
        .iter()
        .find(|d| d.path.contains("cover/test_small.jpg"))
        .expect("detail for cover/test_small.jpg missing");
    assert_eq!(detail_cover_small.action, "skipped");
    assert_eq!(detail_cover_small.new_width, None);
    assert_eq!(detail_cover_small.new_height, None);
    assert!(detail_cover_small.error.is_none());

    let detail_gallery_large = result
        .details
        .iter()
        .find(|d| d.path.contains("gallery/test_large.jpg"))
        .expect("detail for gallery/test_large.jpg missing");
    assert_eq!(detail_gallery_large.action, "resized");

    // ── 9. Assert empty subdirs produce no entries ───────────────────────────
    // detail/ and sku/ are empty → should not appear in details
    let detail_paths: Vec<&str> = result.details.iter().map(|d| d.path.as_str()).collect();
    assert!(
        !detail_paths.iter().any(|p| p.contains("detail/")),
        "empty detail/ should not produce resize entries"
    );
    assert!(
        !detail_paths.iter().any(|p| p.contains("sku/")),
        "empty sku/ should not produce resize entries"
    );

    Ok(())
}
