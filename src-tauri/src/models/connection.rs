// EGrab - Core Data Models: Connection
// Derived from: docs/protocols/data-models.md v1.0.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConnectionInfo {
    pub port: u16,
    pub endpoint: String,
    pub browser_version: String,
    pub state: ConnectionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { browser_version: String },
    Reconnecting { attempt: u8 },
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    pub tab_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CdpEndpoint {
    pub port: u16,
    pub endpoint: String,
    pub browser_version: Option<String>,
}
