// EGrab - Image Downloader Module
// Bulk image download with concurrency control, retry logic, and URL cleaning.
// Derived from: src/protocols/downloader.ts, PRD 3.1.3

pub mod image;

pub use image::{DownloadBatchResult, DownloadImageInput, DownloadImageResult, ImageDownloader};
