// EGrab - Core Data Models: Product
// Derived from: docs/protocols/data-models.md v1.0.0
// Field names must match PRD 3.1.2 exactly

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductData {
    pub title: String,
    pub cover: ImageRef,
    pub gallery: Vec<ImageRef>,
    pub description: Description,
    pub detail_images: Vec<ImageRef>,
    pub skus: Vec<SkuItem>,
    pub sku_images: HashMap<String, ImageRef>,
    pub price: PriceRange,
    pub shop: ShopInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImageRef {
    pub original_url: String,
    pub thumbnail_url: String,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkuItem {
    pub name: String,
    pub value: String,
    pub price: f64,
    pub stock: Option<u32>,
    pub image: Option<ImageRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PriceRange {
    pub min_price: f64,
    pub max_price: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShopInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Description {
    pub text: String,
    pub html: Option<String>,
    pub specs: Vec<SpecItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SpecItem {
    pub key: String,
    pub value: String,
}

/// Result of a resize operation on a single image.
/// Output goes to `proportioned/` subdirectory; originals are never modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// Detail of a single image resize operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResizeDetail {
    /// Image file path
    pub path: String,
    /// Original width in pixels
    pub original_width: u32,
    /// Original height in pixels
    pub original_height: u32,
    /// New width after resize (None if skipped)
    pub new_width: Option<u32>,
    /// New height after resize (None if skipped)
    pub new_height: Option<u32>,
    /// Action taken: "resized", "skipped", or "failed"
    pub action: String,
    /// Error message (only present when action="failed")
    pub error: Option<String>,
}
