// EGrab - IPC Commands: Config
// get_config / set_config Tauri command implementations.

use crate::config::AppConfigManager;
use crate::models::{AppConfig, IpcError};

/// Returns the current application configuration.
///
/// Corresponds to protocol: GetConfigCommand
/// - name: `get_config`
/// - params: none
/// - returns: `AppConfig`
#[tauri::command]
pub async fn get_config(
    state: tauri::State<'_, AppConfigManager>,
) -> Result<AppConfig, IpcError> {
    Ok(state.get_config())
}

/// Saves (validates and persists) a new application configuration.
///
/// Corresponds to protocol: SetConfigCommand
/// - name: `set_config`
/// - params: `{ config: AppConfig }`
/// - returns: `true` on success
/// - errors: `CONFIG_INVALID` if validation fails
#[tauri::command]
pub async fn set_config(
    state: tauri::State<'_, AppConfigManager>,
    config: AppConfig,
) -> Result<bool, IpcError> {
    state.set_config(config)
}
