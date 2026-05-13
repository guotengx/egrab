// EGrab - IPC Commands: CDP
// cdp_connect / cdp_disconnect / cdp_status / cdp_list_tabs
// Tauri command implementations.
//
// Protocol reference: src/protocols/ipc-commands.ts

use crate::cdp::CdpManager;
use crate::models::{ConnectionInfo, ConnectionState, IpcError, TabInfo};

/// Connects to a local CDP browser on 127.0.0.1:{port}.
///
/// Corresponds to protocol: CdpConnectCommand
/// - name: `cdp_connect`
/// - params: `{ port: number }`
/// - returns: `ConnectionInfo`
/// - errors: `CDP_CONNECT_FAILED` | `CDP_TIMEOUT`
#[tauri::command]
pub async fn cdp_connect(
    state: tauri::State<'_, CdpManager>,
    port: u16,
) -> Result<ConnectionInfo, IpcError> {
    state.connect(port).await
}

/// Disconnects from the current CDP browser session.
///
/// Corresponds to protocol: CdpDisconnectCommand
/// - name: `cdp_disconnect`
/// - params: none
/// - returns: `true` if a connection was active, `false` if already disconnected
#[tauri::command]
pub async fn cdp_disconnect(
    state: tauri::State<'_, CdpManager>,
) -> Result<bool, IpcError> {
    state.disconnect().await
}

/// Returns the current CDP connection state.
///
/// Corresponds to protocol: CdpStatusCommand
/// - name: `cdp_status`
/// - params: none
/// - returns: `ConnectionState`
#[tauri::command]
pub async fn cdp_status(
    state: tauri::State<'_, CdpManager>,
) -> Result<ConnectionState, IpcError> {
    Ok(state.status().await)
}

/// Lists all open browser tabs/pages.
///
/// Corresponds to protocol: CdpListTabsCommand
/// - name: `cdp_list_tabs`
/// - params: none
/// - returns: `TabInfo[]`
/// - errors: `CDP_CONNECT_FAILED` if not connected
#[tauri::command]
pub async fn cdp_list_tabs(
    state: tauri::State<'_, CdpManager>,
) -> Result<Vec<TabInfo>, IpcError> {
    state.list_tabs().await
}

/// Navigates the connected browser to a given URL.
///
/// If no active page exists, a new blank page is created first.
/// Page load timeout is 30s.
///
/// Corresponds to protocol extension (not in ARCHITECTURE 5.1):
/// - name: `cdp_navigate`
/// - params: `{ url: string }`
/// - returns: nothing on success
/// - errors: `CDP_CONNECT_FAILED` | `CDP_TIMEOUT`
#[tauri::command]
pub async fn cdp_navigate(
    state: tauri::State<'_, CdpManager>,
    url: String,
) -> Result<(), IpcError> {
    state.navigate(&url).await
}

/// Automatically detects and connects to CDP on localhost.
///
/// Corresponds to protocol: CdpAutoConnectCommand
/// - name: `cdp_auto_connect`
/// - params: none
/// - returns: `ConnectionInfo`
/// - errors: `NO_BROWSER_FOUND` | `CDP_LAUNCH_TIMEOUT` | `CDP_CONNECT_FAILED` | `CDP_TIMEOUT`
#[tauri::command]
pub async fn cdp_auto_connect(
    state: tauri::State<'_, CdpManager>,
) -> Result<ConnectionInfo, IpcError> {
    state.auto_connect().await
}
