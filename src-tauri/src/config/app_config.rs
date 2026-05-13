// EGrab - Config Module: AppConfigManager
// In-memory configuration manager with validation.
// Persistence to disk not implemented (MVP: memory-only).

use crate::models::{AppConfig, BrowserLaunchCommand, BrowserOs, BrowserType, ErrorCode, IpcError};

/// Manages application configuration in memory with thread-safe access.
pub struct AppConfigManager {
    inner: std::sync::Mutex<AppConfig>,
}

impl AppConfigManager {
    /// Creates a new AppConfigManager with default configuration.
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(Self::default_config()),
        }
    }

    /// Returns the current configuration (cloned).
    pub fn get_config(&self) -> AppConfig {
        // Lock is held only for clone; no long critical section.
        // expect is safe here: mutex poisoning only occurs if another thread
        // panicked while holding the lock, which cannot happen in this
        // single-accessor scenario (all access is through Tauri commands
        // on the main thread or async runtime).
        self.inner
            .lock()
            .expect(
                "AppConfigManager mutex poisoned (should never occur in single-accessor pattern)",
            )
            .clone()
    }

    /// Validates and saves new configuration.
    /// Returns Ok(true) on success, or IpcError with code CONFIG_INVALID on validation failure.
    pub fn set_config(&self, config: AppConfig) -> Result<bool, IpcError> {
        // --- Validation ---
        if config.cdp_port == 0 {
            return Err(IpcError {
                code: ErrorCode::ConfigInvalid,
                message: "cdp_port must be in range 1-65535".to_string(),
                recoverable: true,
                step: None,
                details: Some(serde_json::json!({
                    "field": "cdp_port",
                    "value": config.cdp_port,
                    "reason": "port cannot be 0"
                })),
            });
        }
        // u16 already enforces 1..=65535 implicitly once we reject 0

        if config.image_concurrency < 1 || config.image_concurrency > 10 {
            return Err(IpcError {
                code: ErrorCode::ConfigInvalid,
                message: format!(
                    "image_concurrency must be in range 1-10, got {}",
                    config.image_concurrency
                ),
                recoverable: true,
                step: None,
                details: Some(serde_json::json!({
                    "field": "image_concurrency",
                    "value": config.image_concurrency,
                    "reason": "must be between 1 and 10"
                })),
            });
        }

        if config.storage_root.trim().is_empty() {
            return Err(IpcError {
                code: ErrorCode::ConfigInvalid,
                message: "storage_root must not be empty".to_string(),
                recoverable: true,
                step: None,
                details: Some(serde_json::json!({
                    "field": "storage_root",
                    "reason": "must not be empty"
                })),
            });
        }

        // --- Persist ---
        // expect is safe here: same reasoning as get_config() — single-accessor pattern.
        let mut guard = self.inner.lock().expect(
            "AppConfigManager mutex poisoned (should never occur in single-accessor pattern)",
        );
        *guard = config;

        Ok(true)
    }

    /// Returns the default AppConfig for the running platform.
    /// Generates browser launch commands for both macOS and Windows
    /// so the settings page can display reference commands.
    fn default_config() -> AppConfig {
        AppConfig {
            cdp_port: 9222,
            storage_root: Self::default_storage_root(),
            image_concurrency: 3,
            browser_launch_commands: Self::default_browser_launch_commands(),
        }
    }

    /// Default storage root based on the current OS.
    fn default_storage_root() -> String {
        #[cfg(target_os = "macos")]
        {
            // Fallback to home directory if HOME is not set.
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/Unknown".to_string());
            format!("{}/EGrab/", home.trim_end_matches('/'))
        }

        #[cfg(target_os = "windows")]
        {
            let userprofile =
                std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            format!("{}\\EGrab\\", userprofile.trim_end_matches('\\'))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            // Non-macOS/Windows fallback (should not normally be reached).
            "~/EGrab/".to_string()
        }
    }

    /// Default browser launch commands for all supported OS/browser combos.
    fn default_browser_launch_commands() -> Vec<BrowserLaunchCommand> {
        vec![
            // macOS
            BrowserLaunchCommand {
                os: BrowserOs::Macos,
                browser: BrowserType::Chrome,
                command: "open -a \"Google Chrome\" --args --remote-debugging-port=9222"
                    .to_string(),
            },
            BrowserLaunchCommand {
                os: BrowserOs::Macos,
                browser: BrowserType::Edge,
                command: "open -a \"Microsoft Edge\" --args --remote-debugging-port=9222"
                    .to_string(),
            },
            // Windows
            BrowserLaunchCommand {
                os: BrowserOs::Windows,
                browser: BrowserType::Chrome,
                command: "chrome.exe --remote-debugging-port=9222".to_string(),
            },
            BrowserLaunchCommand {
                os: BrowserOs::Windows,
                browser: BrowserType::Edge,
                command: "msedge.exe --remote-debugging-port=9222".to_string(),
            },
        ]
    }
}

// Note: no manual From<IpcError> for InvokeError needed.
// Tauri 2 provides a blanket `impl<T: Serialize> From<T> for InvokeError`,
// so IpcError (which derives Serialize) is automatically convertible.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_config() -> AppConfig {
        AppConfig {
            cdp_port: 9222,
            storage_root: "/tmp/egrab-test/".to_string(),
            image_concurrency: 3,
            browser_launch_commands: vec![],
        }
    }

    #[test]
    fn test_default_config_has_valid_port() {
        let cfg = AppConfigManager::default_config();
        assert!(cfg.cdp_port > 0, "cdp_port should be in valid range");
        assert!(
            (1..=3).contains(&cfg.image_concurrency),
            "image_concurrency should default to 3"
        );
        assert!(
            !cfg.storage_root.trim().is_empty(),
            "storage_root should not be empty"
        );
        assert!(
            cfg.browser_launch_commands.len() >= 2,
            "should have at least 2 browser launch commands"
        );
    }

    #[test]
    fn test_get_config_returns_default() {
        let mgr = AppConfigManager::new();
        let cfg = mgr.get_config();
        assert_eq!(cfg.cdp_port, 9222);
        assert_eq!(cfg.image_concurrency, 3);
    }

    #[test]
    fn test_set_config_valid() {
        let mgr = AppConfigManager::new();
        let mut cfg = make_valid_config();
        cfg.image_concurrency = 5;

        let result = mgr.set_config(cfg);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let stored = mgr.get_config();
        assert_eq!(stored.image_concurrency, 5);
    }

    #[test]
    fn test_set_config_rejects_zero_port() {
        let mgr = AppConfigManager::new();
        let mut cfg = make_valid_config();
        cfg.cdp_port = 0;

        let result = mgr.set_config(cfg);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::ConfigInvalid));
        assert!(err.message.contains("cdp_port"));
    }

    #[test]
    fn test_set_config_rejects_invalid_concurrency() {
        let mgr = AppConfigManager::new();

        let mut cfg = make_valid_config();
        cfg.image_concurrency = 0;
        assert!(mgr.set_config(cfg).is_err());

        let mut cfg = make_valid_config();
        cfg.image_concurrency = 11;
        assert!(mgr.set_config(cfg).is_err());

        // Edge values should pass
        let mut cfg = make_valid_config();
        cfg.image_concurrency = 1;
        assert!(mgr.set_config(cfg).is_ok());

        let mut cfg = make_valid_config();
        cfg.image_concurrency = 10;
        assert!(mgr.set_config(cfg).is_ok());
    }

    #[test]
    fn test_set_config_rejects_empty_storage_root() {
        let mgr = AppConfigManager::new();
        let mut cfg = make_valid_config();
        cfg.storage_root = "   ".to_string();

        let result = mgr.set_config(cfg);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("storage_root"));
    }

    #[test]
    fn test_set_config_persists_change() {
        let mgr = AppConfigManager::new();
        let mut cfg = make_valid_config();
        cfg.cdp_port = 9223;
        mgr.set_config(cfg).unwrap();

        assert_eq!(mgr.get_config().cdp_port, 9223);
    }
}
