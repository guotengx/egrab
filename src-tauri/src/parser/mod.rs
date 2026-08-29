// EGrab - Parser Module
// 平台解析器注册表。
//
// 自 v2 起，所有平台解析逻辑均由 **外置规则包** 驱动
// （见 src-tauri/rules/ 与运行时的 <app_data>/com.egrab.app/rules/），
// 不再有任何平台专属的 Rust 代码。平台改版时只需修改规则文件，
// 无需重新编译或重装程序。
//
// Derived from: src/protocols/parser.ts, ARCHITECTURE 4.3

pub mod rules;
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

/// Returns all platform parsers defined by the currently active rule pack.
pub fn all_parsers() -> Vec<Box<dyn PlatformParser>> {
    let (pack, _source) = rules::load_rule_pack();
    pack.platforms
        .into_iter()
        .map(|rule| Box::new(rules::RuleParser::new(rule)) as Box<dyn PlatformParser>)
        .collect()
}

/// Finds the first parser that can handle the given URL.
pub fn find_parser(url: &str) -> Option<Box<dyn PlatformParser>> {
    all_parsers().into_iter().find(|p| p.can_handle(url))
}
