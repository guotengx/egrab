// EGrab - Core Data Models
// Module entry point - declares public interface

pub mod config;
pub mod connection;
pub mod product;
pub mod task;

pub use config::{
    AppConfig, BrowserLaunchCommand, BrowserOs, BrowserType, MetaJsonDocument, RawJsonDocument,
};
pub use connection::{CdpEndpoint, ConnectionInfo, ConnectionState, TabInfo};
pub use product::{Description, ImageRef, PriceRange, ProductData, ShopInfo, SkuItem, SpecItem};
pub use task::{
    DuplicateTaskConflict, ErrorCode, ImageIndexInput, ImageRecord, ImageType, IpcError,
    ScrapeErrorInfo, ScrapeStep, Task, TaskDetail, TaskFilter, TaskResult, TaskStatus, TaskSummary,
    TaskUpdate,
};
