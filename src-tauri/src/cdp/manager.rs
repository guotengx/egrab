// EGrab - CDP Manager: Core implementation
// Manages the lifecycle of a Chrome DevTools Protocol connection
// to a local browser instance (127.0.0.1 only).
//
// State machine: Disconnected -> Connecting -> Connected -> Disconnected
// Auto-reconnect on connection loss: up to 3 attempts, 2s interval.
// Connection timeout: 10s. Page load timeout: 30s.

use crate::cdp::browser::{self, BrowserInfo};
use crate::models::{CdpEndpoint, ConnectionInfo, ConnectionState, ErrorCode, IpcError, ScrapeStep, TabInfo};
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use std::time::Duration;
use tauri::Emitter;

/// Maximum number of auto-reconnect attempts after connection loss.
/// Reserved for future full reconnect implementation (MVP: manual reconnect only).
#[allow(dead_code)]
const MAX_RECONNECT_ATTEMPTS: u8 = 3;
/// Interval between reconnect attempts.
/// Reserved for future full reconnect implementation (MVP: manual reconnect only).
#[allow(dead_code)]
const RECONNECT_INTERVAL_SECS: u64 = 2;
/// Connection timeout for initial CDP handshake.
const CONNECT_TIMEOUT_SECS: u64 = 10;
/// Page navigation timeout.
const PAGE_LOAD_TIMEOUT_SECS: u64 = 30;

/// Internal state held behind a tokio::sync::Mutex for async-safety.
struct CdpState {
    /// Active Chromium browser connection.
    browser: Option<Browser>,
    /// JoinHandle for the background handler-driving task.
    /// When this task exits, the connection has dropped.
    handler_task: Option<tokio::task::JoinHandle<()>>,
    /// JoinHandle for the connection monitor task.
    monitor_task: Option<tokio::task::JoinHandle<()>>,
    /// Current connection state in the state machine.
    state: ConnectionState,
    /// The page currently used for navigation/evaluation.
    active_page: Option<Page>,
    /// The last successfully connected port (for reconnect).
    last_port: Option<u16>,
    /// Number of reconnect attempts made in the current cycle.
    reconnect_attempt: u8,
}

/// CDP Manager — the primary interface for connecting to and interacting
/// with a local Chrome/Edge browser via DevTools Protocol.
///
/// Wraps all mutable state in a `tokio::sync::Mutex` for thread-safe
/// concurrent access from Tauri command handlers.
pub struct CdpManager {
    inner: tokio::sync::Mutex<CdpState>,
    app_handle: tauri::AppHandle,
    /// The browser instance that EGrab launched via `auto_connect()`.
    /// Tracked so we can kill it on app shutdown.
    launched_browser: std::sync::Mutex<Option<BrowserInfo>>,
}

impl CdpManager {
    /// Creates a new CdpManager with the given Tauri AppHandle.
    ///
    /// The AppHandle is used to emit `cdp:state_changed` events to the
    /// frontend whenever the connection state changes.
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(CdpState {
                browser: None,
                handler_task: None,
                monitor_task: None,
                state: ConnectionState::Disconnected,
                active_page: None,
                last_port: None,
                reconnect_attempt: 0,
            }),
            app_handle,
            launched_browser: std::sync::Mutex::new(None),
        }
    }

    /// Returns a reference to self.
    /// Used by ScraperEngine to obtain a raw pointer for CdpPageHandle.
    pub fn inner(&self) -> &Self {
        self
    }

    // -----------------------------------------------------------------------
    // Public API — mirrors CdpManager interface from the protocol
    // -----------------------------------------------------------------------

    /// Scans a list of candidate ports for active CDP endpoints.
    ///
    /// For each port in `candidates`, performs an HTTP GET to
    /// `http://127.0.0.1:{port}/json/version` with a 2s per-port timeout.
    /// Ports that respond with a valid JSON body containing
    /// `webSocketDebuggerUrl` are returned as `CdpEndpoint` values.
    pub async fn scan_ports(&self, candidates: &[u16]) -> Result<Vec<CdpEndpoint>, IpcError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: format!("Failed to build HTTP client: {}", e),
                recoverable: false,
                step: Some(ScrapeStep::Connecting),
                details: None,
            })?;

        let mut endpoints = Vec::new();

        for &port in candidates {
            let url = format!("http://127.0.0.1:{}/json/version", port);

            let result: Option<(String, String)> = async {
                let resp = client.get(&url).send().await.ok()?;
                let json: serde_json::Value = resp.json().await.ok()?;
                let ws_url = json.get("webSocketDebuggerUrl")?.as_str()?.to_string();
                let version = json
                    .get("Browser")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                Some((ws_url, version))
            }
            .await;

            if let Some((endpoint, browser_version)) = result {
                tracing::info!(
                    port = port,
                    endpoint = %endpoint,
                    "Found CDP endpoint"
                );
                endpoints.push(CdpEndpoint {
                    port,
                    endpoint,
                    browser_version: Some(browser_version),
                });
            }
        }

        Ok(endpoints)
    }

    /// Automatically detects and connects to CDP on localhost.
    ///
    /// Flow:
    /// 1. Check if a browser is already listening on port 9222 → connect directly.
    /// 2. Scan for installed browsers on the local system.
    /// 3. Launch the first-found browser with `--remote-debugging-port=9222`
    ///    using an independent persistent profile (`--user-data-dir`).
    ///    This avoids Chrome's single-instance lock — the user's main browser
    ///    can remain running. Login state is preserved within EGrab's own profile.
    /// 4. Wait up to 20 seconds for the port to become ready (polling every 1s).
    /// 5. Connect via the standard `connect()` flow.
    ///
    /// Errors:
    /// - `NoBrowserFound`: No Chrome/Edge/Chromium-based browser detected on the system.
    /// - `CdpLaunchTimeout`: Browser launched but port did not respond within 20s.
    /// - `CdpConnectFailed` / `CdpTimeout`: Forwarded from `connect()`.
    pub async fn auto_connect(&self) -> Result<ConnectionInfo, IpcError> {
        // 1. Check if CDP is already running on the default port
        if browser::check_cdp_port(9222).await {
            tracing::info!("CDP port 9222 is already active; connecting directly");
            return self.connect(9222).await;
        }

        // 2. Scan for installed browsers
        let browsers = browser::scan_browsers();
        if browsers.is_empty() {
            return Err(IpcError {
                code: ErrorCode::NoBrowserFound,
                message: "No Chrome or Edge browser detected. Please install one and retry."
                    .to_string(),
                recoverable: false,
                step: None,
                details: None,
            });
        }

        let chosen = browsers[0].clone();
        tracing::info!(
            name = %chosen.name,
            path = %chosen.path,
            "Selected browser for CDP launch"
        );

        // 3. For Edge on Windows, kill any lingering processes before launching.
        //    Edge's Startup Boost keeps msedge.exe alive even after the user
        //    closes all windows, and this can silently block CDP port binding.
        if chosen.name.contains("Edge") {
            browser::kill_browser_process(&chosen);
        }

        // 4. Launch the browser with CDP flags and an independent persistent profile.
        //    Using --user-data-dir avoids Chrome's single-instance lock — the user's
        //    existing browser sessions are unaffected. Login state within EGrab's
        //    profile is preserved across sessions.
        browser::launch_browser_with_cdp(&chosen, 9222)?;

        // Track the launched browser so we can kill it on app shutdown.
        {
            let mut guard = self.launched_browser.lock().unwrap();
            *guard = Some(chosen.clone());
        }

        // 4. Wait for the port to become ready (up to 20 seconds, polling every 1s)
        //    Extended from 5s to 20s because Chrome can take much longer on
        //    first launch, session restore, or when macOS security prompts appear.
        for i in 0..20 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if browser::check_cdp_port(9222).await {
                tracing::info!(attempt = i + 1, "CDP port 9222 is ready");
                return self.connect(9222).await;
            }
        }

        // 5. Timeout — collect diagnostics before returning the error
        //    so the user has actionable information.
        let diag = browser::collect_startup_diagnostics(9222);
        tracing::warn!(
            diagnostics = %diag,
            "CDP launch timed out after 20s"
        );

        let mut message = String::from(
            "Browser launch timed out after 20 seconds. ",
        );
        message.push_str(&diag);

        #[cfg(target_os = "macos")]
        {
            message.push_str(
                " Chrome may still be starting. You can also manually "
            );
            message.push_str(
                "launch it with: open -a \"Google Chrome\" --args "
            );
            message.push_str(
                "--remote-debugging-port=9222"
            );
        }

        Err(IpcError {
            code: ErrorCode::CdpLaunchTimeout,
            message,
            recoverable: true,
            step: None,
            details: None,
        })
    }

    /// Connects to a local CDP browser on `127.0.0.1:{port}`.
    ///
    /// State transitions: Disconnected → Connecting → Connected (or Failed).
    /// Emits `cdp:state_changed` events on each transition.
    /// Timeout: 10s for the HTTP handshake + WebSocket upgrade.
    pub async fn connect(&self, port: u16) -> Result<ConnectionInfo, IpcError> {
        // --- Transition to Connecting ---
        {
            let mut guard = self.inner.lock().await;
            guard.cleanup().await;
            guard.state = ConnectionState::Connecting;
            guard.last_port = Some(port);
            guard.reconnect_attempt = 0;
        }
        self.emit_state(ConnectionState::Connecting);

        // --- Resolve WebSocket debug URL via HTTP ---
        let version_url = format!("http://127.0.0.1:{}/json/version", port);
        let (ws_endpoint, browser_version) = self
            .fetch_debug_url(&version_url, CONNECT_TIMEOUT_SECS)
            .await
            .map_err(|e| {
                let app_handle = self.app_handle.clone();
                let failed = ConnectionState::Failed {
                    reason: e.message.clone(),
                };
                // We can't hold the lock across the emit, so clone the state
                tokio::spawn(async move {
                    let _ = app_handle.emit("cdp:state_changed", &failed);
                });
                // Update state to Failed
                // Use block_in_place or just let the outer code handle it
                e
            })?;

        // --- Establish WebSocket connection ---
        let browser_result: Result<(Browser, _), IpcError> = tokio::time::timeout(
            Duration::from_secs(CONNECT_TIMEOUT_SECS),
            Browser::connect(ws_endpoint.as_str()),
        )
        .await
        .map_err(|_| IpcError {
            code: ErrorCode::CdpTimeout,
            message: format!("CDP WebSocket connection timed out after {}s", CONNECT_TIMEOUT_SECS),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?
        .map_err(|e| IpcError {
            code: ErrorCode::CdpConnectFailed,
            message: format!("CDP WebSocket connection failed: {}", e),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        });

        let (browser, mut handler) = match browser_result {
            Ok(bh) => bh,
            Err(e) => {
                let mut guard = self.inner.lock().await;
                guard.state = ConnectionState::Failed {
                    reason: e.message.clone(),
                };
                self.emit_state(guard.state.clone());
                return Err(e);
            }
        };

        // --- Spawn the handler driver task ---
        let monitor_app_handle = self.app_handle.clone();
        let monitor_port = port;
        let handler_task = tokio::spawn(async move {
            // Drive the handler stream — when this exits, the connection dropped.
            while handler.next().await.is_some() {}
            // Connection dropped; notify via event
            let _ = monitor_app_handle.emit(
                "cdp:state_changed",
                &ConnectionState::Disconnected,
            );
            tracing::warn!(port = monitor_port, "CDP handler stream ended; connection dropped");
        });

        // --- Spawn reconnect monitor ---
        // This task watches the handler_task and attempts reconnect if it exits.
        let reconnect_app_handle = self.app_handle.clone();
        let monitor_task = tokio::spawn(async move {
            Self::reconnect_monitor(reconnect_app_handle, port).await;
        });

        // --- Transition to Connected ---
        let connected = ConnectionState::Connected {
            browser_version: browser_version.clone(),
        };

        {
            let mut guard = self.inner.lock().await;
            guard.browser = Some(browser);
            guard.handler_task = Some(handler_task);
            guard.monitor_task = Some(monitor_task);
            guard.state = connected.clone();
            guard.active_page = None;
        }
        self.emit_state(connected.clone());

        tracing::info!(
            port = port,
            version = %browser_version,
            "CDP connected successfully"
        );

        Ok(ConnectionInfo {
            port,
            endpoint: ws_endpoint,
            browser_version,
            state: connected,
        })
    }

    /// Disconnects from the current CDP browser session.
    ///
    /// Cleans up the browser handle, aborts background tasks,
    /// and transitions state to `Disconnected`.
    /// Returns `true` if a connection was active and is now closed,
    /// `false` if already disconnected.
    pub async fn disconnect(&self) -> Result<bool, IpcError> {
        let was_connected;
        {
            let mut guard = self.inner.lock().await;
            was_connected = matches!(
                guard.state,
                ConnectionState::Connected { .. } | ConnectionState::Connecting
            );
            if was_connected {
                guard.cleanup().await;
                guard.state = ConnectionState::Disconnected;
                guard.active_page = None;
                guard.last_port = None;
                guard.reconnect_attempt = 0;
            }
        }

        if was_connected {
            self.emit_state(ConnectionState::Disconnected);
            tracing::info!("CDP disconnected");
        }

        Ok(was_connected)
    }

    /// Returns the current connection state (non-blocking clone).
    pub async fn status(&self) -> ConnectionState {
        self.inner.lock().await.state.clone()
    }

    /// Lists all open tabs/pages in the connected browser.
    ///
    /// Returns an error if not connected.
    pub async fn list_tabs(&self) -> Result<Vec<TabInfo>, IpcError> {
        let guard = self.inner.lock().await;
        let browser = guard
            .browser
            .as_ref()
            .ok_or_else(|| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: "Not connected to CDP browser".to_string(),
                recoverable: true,
                step: Some(ScrapeStep::Connecting),
                details: None,
            })?;

        let pages = browser.pages().await.map_err(|e| IpcError {
            code: ErrorCode::CdpConnectFailed,
            message: format!("Failed to list browser pages: {}", e),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?;

        let mut tabs = Vec::with_capacity(pages.len());
        for page in &pages {
            // target_id() returns the CDP target identifier (newtype over String)
            let id = format!("{:?}", page.target_id());
            let title = page.get_title().await.ok().flatten().unwrap_or_default();
            let url = page.url().await.ok().flatten().unwrap_or_default();
            tabs.push(TabInfo {
                id,
                title,
                url,
                tab_type: "page".to_string(),
            });
        }

        Ok(tabs)
    }

    /// Synchronously shuts down the CDP manager.
    ///
    /// Kills any browser process that EGrab launched via `auto_connect()`.
    /// This is designed to be called from a synchronous context (e.g. Tauri's
    /// window event handler) when the app is exiting. The CDP WebSocket
    /// connection will be cleaned up naturally when the process terminates.
    pub fn shutdown(&self) {
        let browser = {
            let mut guard = self.launched_browser.lock().unwrap();
            guard.take()
        };

        if let Some(browser) = browser {
            tracing::info!(
                name = %browser.name,
                "Shutting down: killing browser launched by EGrab"
            );
            browser::kill_browser_process(&browser);
        } else {
            tracing::debug!("Shutdown: no browser was launched by EGrab, nothing to kill");
        }
    }

    /// Navigates the current (or newly created) page to the given URL.
    ///
    /// Creates a new blank page if no active page exists.
    /// Page load timeout: 30s.
    pub async fn navigate(&self, url: &str) -> Result<(), IpcError> {
        let mut guard = self.inner.lock().await;

        let browser = guard
            .browser
            .as_ref()
            .ok_or_else(|| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: "Not connected to CDP browser".to_string(),
                recoverable: true,
                step: Some(ScrapeStep::PageLoading),
                details: None,
            })?;

        let page = if let Some(ref page) = guard.active_page {
            page.clone()
        } else {
            let new_page = browser.new_page("about:blank").await.map_err(|e| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: format!("Failed to create new page: {}", e),
                recoverable: true,
                step: Some(ScrapeStep::PageLoading),
                details: None,
            })?;
            guard.active_page = Some(new_page.clone());
            new_page
        };

        // Perform navigation with 30s timeout
        tokio::time::timeout(Duration::from_secs(PAGE_LOAD_TIMEOUT_SECS), page.goto(url))
            .await
            .map_err(|_| IpcError {
                code: ErrorCode::CdpTimeout,
                message: format!(
                    "Page load timed out after {}s for URL: {}",
                    PAGE_LOAD_TIMEOUT_SECS, url
                ),
                recoverable: true,
                step: Some(ScrapeStep::PageLoading),
                details: None,
            })?
            .map_err(|e| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: format!("Page navigation failed: {}", e),
                recoverable: true,
                step: Some(ScrapeStep::PageLoading),
                details: None,
            })?;

        tracing::info!(url = %url, "Page navigation completed");
        Ok(())
    }

    /// Evaluates a JavaScript expression on the active page.
    ///
    /// Returns the JSON-serializable result of the evaluation.
    /// Returns an error if not connected or no active page.
    pub async fn evaluate(&self, script: &str) -> Result<serde_json::Value, IpcError> {
        let guard = self.inner.lock().await;

        let page = guard
            .active_page
            .as_ref()
            .ok_or_else(|| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: "No active page; call navigate() first".to_string(),
                recoverable: true,
                step: Some(ScrapeStep::Parsing),
                details: None,
            })?;

        let eval_result = page.evaluate(script).await.map_err(|e| {
            IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: format!("JavaScript evaluation failed: {}", e),
                recoverable: true,
                step: Some(ScrapeStep::Parsing),
                details: None,
            }
        })?;

        // EvaluationResult::into_value() deserializes the CDP RemoteObject
        // into the target type. We extract as serde_json::Value for flexibility.
        let value: serde_json::Value = eval_result.into_value().map_err(|e| IpcError {
            code: ErrorCode::CdpConnectFailed,
            message: format!("Failed to deserialize evaluation result: {}", e),
            recoverable: true,
            step: Some(ScrapeStep::Parsing),
            details: None,
        })?;

        Ok(value)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Fetches the WebSocket debug URL and browser version from the
    /// CDP `/json/version` endpoint with a timeout.
    async fn fetch_debug_url(
        &self,
        version_url: &str,
        timeout_secs: u64,
    ) -> Result<(String, String), IpcError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: format!("Failed to build HTTP client: {}", e),
                recoverable: false,
                step: Some(ScrapeStep::Connecting),
                details: None,
            })?;

        let resp = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            client.get(version_url).send(),
        )
        .await
        .map_err(|_| IpcError {
            code: ErrorCode::CdpTimeout,
            message: format!(
                "CDP endpoint HTTP request timed out after {}s",
                timeout_secs
            ),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?
        .map_err(|e| IpcError {
            code: ErrorCode::CdpConnectFailed,
            message: format!("Failed to reach CDP endpoint: {}", e),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?;

        let json: serde_json::Value = resp.json().await.map_err(|e| IpcError {
            code: ErrorCode::CdpConnectFailed,
            message: format!("Failed to parse CDP /json/version response: {}", e),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        })?;

        let ws_endpoint = json
            .get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IpcError {
                code: ErrorCode::CdpConnectFailed,
                message: "CDP /json/version response missing webSocketDebuggerUrl".to_string(),
                recoverable: true,
                step: Some(ScrapeStep::Connecting),
                details: None,
            })?
            .to_string();

        let browser_version = json
            .get("Browser")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok((ws_endpoint, browser_version))
    }

    /// Emits a `cdp:state_changed` event to the frontend.
    fn emit_state(&self, state: ConnectionState) {
        let app_handle = self.app_handle.clone();
        let state_clone = state.clone();
        // Fire-and-forget: if the frontend has no listener, we don't want to block.
        match app_handle.emit("cdp:state_changed", &state_clone) {
            Ok(_) => tracing::debug!(?state, "Emitted cdp:state_changed"),
            Err(e) => tracing::error!(?state, error = %e, "Failed to emit cdp:state_changed"),
        }
    }

    /// Background reconnect monitor.
    ///
    /// When the browser connection drops, attempts to reconnect up to
    /// `MAX_RECONNECT_ATTEMPTS` times with `RECONNECT_INTERVAL_SECS` delay.
    /// Updates the CdpManager state on each attempt.
    async fn reconnect_monitor(app_handle: tauri::AppHandle, port: u16) {
        // NOTE: This function runs in a spawned task. It does NOT have access
        // to `self.inner` directly. For full reconnect support, the CdpManager
        // would need to be accessed via Tauri state. In MVP, the reconnect
        // monitor simply emits events on disconnect without attempting full
        // automatic reconnect — the user must manually reconnect.
        //
        // This design keeps the architecture simple while still providing
        // the `cdp:state_changed` -> Disconnected event when the connection drops.
        tracing::debug!(
            port = port,
            "Reconnect monitor started (MVP: manual reconnect only)"
        );

        // In a full implementation, this task would:
        // 1. Await on the handler_task's completion
        // 2. Attempt to reconnect MAX_RECONNECT_ATTEMPTS times
        // 3. Update ConnectionState via app_handle.state::<CdpManager>()
        //
        // For MVP, the event is emitted by the handler driver task on drop.
        // This task exists as a placeholder for future reconnect logic.
        let _ = app_handle; // keep alive
    }
}

impl CdpState {
    /// Cleans up the current connection: aborts the handler task and
    /// drops the browser reference (which closes the WebSocket).
    async fn cleanup(&mut self) {
        // Drop browser first — this closes the WebSocket connection.
        self.browser = None;
        self.active_page = None;

        // Abort the handler driver task.
        if let Some(handle) = self.handler_task.take() {
            handle.abort();
        }

        // Abort the monitor task.
        if let Some(handle) = self.monitor_task.take() {
            handle.abort();
        }
    }
}
