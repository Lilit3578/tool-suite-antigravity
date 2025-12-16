//! System integration command module
//!
//! Handles system-level operations like accessibility permissions and logging.

use crate::shared::types::LogRequest;
use crate::system::automation;
use crate::shared::error::AppResult;

/// Get the currently active application name
#[tauri::command]
pub fn get_active_app() -> AppResult<String> {
    automation::get_active_app().map_err(crate::shared::error::AppError::from)
}

/// Check if accessibility permissions are granted (macOS only)
#[tauri::command]
pub async fn check_accessibility_permissions() -> AppResult<bool> {
    #[cfg(target_os = "macos")]
    {
        use crate::system::automation::macos::check_accessibility_permissions;
        Ok(check_accessibility_permissions())
    }
    #[cfg(not(target_os = "macos"))]
    Ok(true)
}

/// Ensure accessibility permissions are granted, prompting user if needed (macOS only)
/// This will trigger the macOS system dialog if permissions are not granted
#[tauri::command]
pub async fn check_accessibility_permission() -> AppResult<bool> {
    #[cfg(target_os = "macos")]
    {
        use crate::system::automation::macos::ensure_accessibility_permissions;
        Ok(ensure_accessibility_permissions())
    }
    #[cfg(not(target_os = "macos"))]
    Ok(true)
}

/// Log a message from the frontend
#[tauri::command]
pub async fn log_message(request: LogRequest) -> AppResult<()> {
    println!("[{}] {}", request.level.to_uppercase(), request.message);
    Ok(())
}
