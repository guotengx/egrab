// EGrab - CDP Connection Management Module
// Module entry point. Provides CdpManager for Chrome DevTools Protocol
// communication with the user's local browser instance.

pub mod browser;
pub mod manager;

pub use manager::CdpManager;
