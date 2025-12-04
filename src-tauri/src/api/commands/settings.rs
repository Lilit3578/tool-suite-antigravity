//! Settings command module
//!
//! Handles application settings persistence.

use crate::shared::settings::AppSettings;
use crate::api::error::CommandResult;

/// Get current application settings
#[tauri::command]
pub async fn get_settings() -> CommandResult<AppSettings> {
    AppSettings::load()
}

/// Save application settings
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> CommandResult<()> {
    settings.save()
}
