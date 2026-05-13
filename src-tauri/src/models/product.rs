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
