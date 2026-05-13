// EGrab - Core Data Models: Config
// Derived from: docs/protocols/data-models.md v1.0.0, docs/protocols/config-interface.md v1.0.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    pub cdp_port: u16,
    pub storage_root: String,
    pub image_concurrency: u32,
    pub browser_launch_commands: Vec<BrowserLaunchCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BrowserLaunchCommand {
    pub os: BrowserOs,
    pub browser: BrowserType,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserOs {
    Macos,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserType {
    Chrome,
    Edge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MetaJsonDocument {
    pub version: String,
    pub platform: String,
    pub item_id: String,
    pub scraped_at: String,
    pub data: super::product::ProductData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RawJsonDocument {
    pub version: String,
    pub platform: String,
    pub item_id: String,
    pub scraped_at: String,
    pub url: String,
    pub raw_data: serde_json::Value,
    pub parser_errors: Vec<super::task::ScrapeErrorInfo>,
}
