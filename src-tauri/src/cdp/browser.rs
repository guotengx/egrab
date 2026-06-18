// EGrab - CDP Browser detection and launch helpers
//
// Scans the local system for CDP-compatible browser installations
// (Chrome, Edge, Chromium, Brave, Arc) and provides utilities for
// launching a browser with the required --remote-debugging-port flag.
//
// Uses a persistent, isolated profile directory so that:
// - The user's main browser can remain running (Chrome's single-instance
//   lock is per-profile; an independent profile avoids the lock).
// - Login state is preserved across sessions within EGrab's own profile.

use crate::models::{ErrorCode, IpcError};
use std::process::Command;

/// Information about a detected browser installation.
#[derive(Clone)]
pub struct BrowserInfo {
    pub name: String,
    pub path: String,
}

/// Scans the local filesystem for installed CDP-compatible browsers.
///
/// Returns a list of detected browsers sorted by priority.
/// macOS: Chrome, Edge, Chromium, Brave, Arc
/// Windows: Chrome, Edge, Brave
pub fn scan_browsers() -> Vec<BrowserInfo> {
    let mut browsers = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            ("Chrome", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            ("Edge", "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
            ("Chromium", "/Applications/Chromium.app/Contents/MacOS/Chromium"),
            ("Brave", "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            ("Arc", "/Applications/Arc.app/Contents/MacOS/Arc"),
        ];
        for (name, path) in &candidates {
            if std::path::Path::new(path).exists() {
                browsers.push(BrowserInfo {
                    name: name.to_string(),
                    path: path.to_string(),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            ("Chrome", r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            ("Chrome (x86)", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
            ("Edge", r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
            ("Edge", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
            ("Brave", r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"),
        ];
        for (name, path) in &candidates {
            if std::path::Path::new(path).exists() {
                browsers.push(BrowserInfo {
                    name: name.to_string(),
                    path: path.to_string(),
                });
            }
        }
    }

    browsers
}

/// Forcefully terminates a running browser process so that a fresh instance
/// can be launched with CDP remote debugging enabled.
///
/// Chrome/Edge only read `--remote-debugging-port` on the first launch of a
/// given profile.  If the browser is already running, subsequent launches
/// ignore the flag.  Killing the existing process and relaunching is the
/// only reliable way to enable CDP while preserving the user's default
/// profile (session restore, bookmarks, extensions, etc.).
/// Forcefully terminates a running browser process.
///
/// Maps the browser's display name to the appropriate platform process name
/// and issues `killall` (macOS) or `taskkill` (Windows).
pub fn kill_browser_process(browser: &BrowserInfo) {
    #[cfg(target_os = "macos")]
    {
        let process_name = match browser.name.as_str() {
            "Chrome" => "Google Chrome",
            "Edge" => "Microsoft Edge",
            "Chromium" => "Chromium",
            "Brave" => "Brave Browser",
            "Arc" => "Arc",
            other => other,
        };
        let result = std::process::Command::new("killall")
            .arg(process_name)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        match result {
            Ok(status) if status.success() => {
                tracing::info!(process = process_name, "Killed existing browser process");
            }
            Ok(status) => {
                tracing::warn!(
                    process = process_name,
                    exit_code = ?status.code(),
                    "killall returned non-zero (process may not be running — non-fatal)"
                );
            }
            Err(e) => {
                tracing::warn!(
                    process = process_name,
                    error = %e,
                    "killall command failed (possible macOS security restriction — non-fatal)"
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let image_name = match browser.name.as_str() {
            "Chrome" | "Chrome (x86)" => "chrome.exe",
            "Edge" => "msedge.exe",
            "Brave" => "brave.exe",
            other => other,
        };

        // Force-kill browser processes (Edge Startup Boost keeps hidden
        // msedge.exe alive even after the user closes all windows, so a
        // single pass may not suffice).
        for pass in 0..3 {
            let result = std::process::Command::new("taskkill")
                .args(["/F", "/IM", image_name])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            match result {
                Ok(status) if status.success() => {
                    tracing::info!(image = image_name, pass, "Killed browser processes");
                }
                Ok(_) => {
                    if pass == 0 {
                        tracing::info!(image = image_name, "No browser processes to kill");
                    }
                    break; // no more processes
                }
                Err(e) => {
                    tracing::warn!(image = image_name, error = %e, "taskkill failed");
                    break;
                }
            }
            // Wait for OS to fully reap processes before retrying / relaunching.
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

/// Returns the path to EGrab's persistent CDP browser profile directory.
///
/// Uses the platform-appropriate application data location so that login
/// state, cookies, and session data are retained across app restarts.
///
/// macOS:   `~/Library/Application Support/com.egrab.app/cdp-profile`
/// Windows: `%APPDATA%\com.egrab.app\cdp-profile`
///
/// The directory is created if it does not already exist.
fn get_cdp_profile_dir() -> String {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/Library/Application Support/com.egrab.app/cdp-profile", home)
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\temp".to_string());
        format!("{}\\com.egrab.app\\cdp-profile", appdata)
    }
}

/// Launches the given browser with CDP remote debugging enabled on `port`.
///
/// **macOS**: Launches the browser binary directly via `browser.path`
/// (e.g. `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`)
/// instead of `open -a`. This ensures that `--remote-debugging-port` is
/// reliably passed to the browser (macOS Launch Services may silently
/// drop arguments passed via `open -a --args`).
///
/// **Uses an independent persistent profile** (`--user-data-dir`) instead of
/// the user's default profile. This avoids Chrome's single-instance lock
/// (the user's main browser can remain running), and EGrab's login state
/// is preserved across sessions within its own profile directory.
/// The user only needs to log in once inside the EGrab-launched browser.
///
/// **Windows**: Launches the browser executable directly via `Command::new`
/// with the CDP flags and the independent profile directory.
pub fn launch_browser_with_cdp(browser: &BrowserInfo, port: u16) -> Result<(), IpcError> {
    let profile_dir = get_cdp_profile_dir();

    // Ensure the profile directory exists.
    std::fs::create_dir_all(&profile_dir).map_err(|e| IpcError {
        code: ErrorCode::CdpLaunchTimeout,
        message: format!("Failed to create CDP profile directory at {}: {}", profile_dir, e),
        recoverable: true,
        step: None,
        details: None,
    })?;

    #[cfg(target_os = "macos")]
    {
        Command::new(&browser.path)
            .arg(format!("--remote-debugging-port={}", port))
            .arg("--remote-allow-origins=*")
            .arg(format!("--user-data-dir={}", profile_dir))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| IpcError {
                code: ErrorCode::CdpLaunchTimeout,
                message: format!("Failed to launch {} at {}: {}", browser.name, browser.path, e),
                recoverable: true,
                step: None,
                details: None,
            })?;
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new(&browser.path);
        cmd.arg(format!("--remote-debugging-port={}", port))
            .arg("--remote-allow-origins=*")
            .arg(format!("--user-data-dir={}", profile_dir))
            .arg("--no-first-run")
            .arg("--no-default-browser-check");
        cmd.spawn().map_err(|e| IpcError {
            code: ErrorCode::CdpLaunchTimeout,
            message: format!("Failed to launch {}: {}", browser.name, e),
            recoverable: true,
            step: None,
            details: None,
        })?;
    }

    Ok(())
}

/// Detects whether any CDP-compatible browser process is currently running.
///
/// Checks for known browser process names via `pgrep` (macOS) or `tasklist` (Windows).
/// Used by `auto_connect()` to determine whether we need to kill a running instance
/// before launching with CDP flags (Chrome's single-instance lock prevents a second
/// instance from using the default profile).
pub fn is_browser_running() -> bool {
    #[cfg(target_os = "macos")]
    {
        // pgrep -fl "Chrome|Edge|Chromium|Brave|Arc"
        match std::process::Command::new("pgrep")
            .args(["-fl", "Chrome|Edge|Chromium|Brave|Arc"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(output) => !output.stdout.is_empty(),
            Err(_) => false,
        }
    }

    #[cfg(target_os = "windows")]
    {
        for exe in &["chrome.exe", "msedge.exe", "brave.exe"] {
            match std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", exe)])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains(exe) {
                        return true;
                    }
                }
                Err(_) => continue,
            }
        }
        false
    }
}

/// Checks whether a CDP-compatible browser is already listening on `127.0.0.1:{port}`.
///
/// Performs an HTTP GET to `/json/version` with a 2-second timeout.
pub async fn check_cdp_port(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/json/version", port);
    reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Collects diagnostic information when CDP startup times out.
///
/// Checks:
/// - Whether browser processes are running (pgrep / tasklist)
/// - Whether the target port is in use (lsof / netstat)
///
/// Returns a single string suitable for inclusion in an error message.
pub fn collect_startup_diagnostics(port: u16) -> String {
    let mut parts: Vec<String> = Vec::new();

    // ── Check browser processes ──────────────────────────────────────
    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("pgrep")
            .args(["-fl", "Chrome|Edge|Chromium|Brave|Arc"])
            .output()
        {
            Ok(output) if !output.stdout.is_empty() => {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                parts.push(format!("Found {} browser process(es) running.", count));
            }
            Ok(_) => {
                parts.push("No browser processes found running.".to_string());
            }
            Err(e) => {
                parts.push(format!("Could not check browser processes: {}", e));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for exe in &["chrome.exe", "msedge.exe", "brave.exe"] {
            match std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", exe)])
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains(exe) {
                        parts.push(format!("Found {} running.", exe));
                    }
                }
                Err(_) => {}
            }
        }
        if parts.is_empty() {
            parts.push("No browser processes found running.".to_string());
        }
    }

    // ── Check port usage ─────────────────────────────────────────────
    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output()
        {
            Ok(output) if !output.stdout.is_empty() => {
                parts.push(format!("Port {} is in use.", port));
            }
            Ok(_) => {
                parts.push(format!(
                    "Port {} is NOT in use (browser may not have started).",
                    port
                ));
            }
            Err(e) => {
                parts.push(format!("Could not check port {}: {}", port, e));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        match std::process::Command::new("netstat")
            .args(["-ano"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let port_str = format!(":{}", port);
                if stdout.contains(&port_str) {
                    parts.push(format!("Port {} is in use.", port));
                } else {
                    parts.push(format!(
                        "Port {} is NOT in use (browser may not have started).",
                        port
                    ));
                }
            }
            Err(e) => {
                parts.push(format!("Could not check port {}: {}", port, e));
            }
        }
    }

    parts.join(" ")
}
