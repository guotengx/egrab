// EGrab - Parser Module
// Platform-specific page parser definitions and registry.
// Derived from: src/protocols/parser.ts, ARCHITECTURE 4.3

pub mod jd;
pub mod taobao;
pub mod utils;

use crate::models::{ProductData, ScrapeErrorInfo};
use async_trait::async_trait;

/// Result of parsing a page.
/// Even if parsing partially fails, raw_data is preserved for debugging.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed product data; may be None if parsing failed completely,
    /// or may contain partial data if some fields were extractable.
    pub product: Option<ProductData>,
    /// Raw extracted data (the result of page.evaluate()).
    pub raw_data: serde_json::Value,
    /// Errors encountered during parsing.
    pub errors: Vec<ScrapeErrorInfo>,
}

/// PageHandle abstracts the CDP-backed page interactions.
/// CDP module will provide a concrete implementation of this trait.
#[async_trait]
pub trait PageHandle: Send + Sync {
    /// Returns the current page URL.
    async fn url(&self) -> anyhow::Result<String>;

    /// Returns the page title.
    async fn title(&self) -> anyhow::Result<String>;

    /// Evaluates a JavaScript expression in the page context.
    /// The script should return a JSON-serializable value.
    async fn evaluate(&self, script: &str) -> anyhow::Result<serde_json::Value>;

    /// Returns the full HTML content of the page (optional, used as fallback).
    async fn content(&self) -> anyhow::Result<String>;
}

/// Trait that all platform parsers must implement.
#[async_trait]
pub trait PlatformParser: Send + Sync {
    /// Returns the platform identifier string (e.g., "taobao", "jd").
    fn platform_id(&self) -> &str;

    /// Checks whether this parser can handle the given URL.
    fn can_handle(&self, url: &str) -> bool;

    /// Extracts the item ID from a URL.
    fn extract_item_id(&self, url: &str) -> anyhow::Result<String>;

    /// Parses a product page and returns structured data.
    /// Returns ParseResult which may contain partial data if some fields failed.
    async fn parse(&self, page: &dyn PageHandle) -> anyhow::Result<ParseResult>;
}

/// Returns all platform parsers registered in the system.
pub fn all_parsers() -> Vec<Box<dyn PlatformParser>> {
    vec![
        Box::new(taobao::TaobaoParser::new()),
        Box::new(jd::JdParser::new()),
    ]
}

/// Finds the first parser that can handle the given URL.
pub fn find_parser(url: &str) -> Option<Box<dyn PlatformParser>> {
    all_parsers().into_iter().find(|p| p.can_handle(url))
}
