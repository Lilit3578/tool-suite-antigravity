//! Settings command module
//!
//! Handles application settings persistence.

use crate::shared::settings::AppSettings;
use crate::shared::error::AppResult;

/// Get current application settings
#[tauri::command]
pub async fn get_settings() -> AppResult<AppSettings> {
    AppSettings::load().await.map_err(|e| crate::shared::error::AppError::Io(e))
}

/// Save application settings
#[tauri::command]
pub async fn save_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> AppResult<()> {
    settings.save(&app_handle).await.map_err(|e| crate::shared::error::AppError::Io(e))
}
