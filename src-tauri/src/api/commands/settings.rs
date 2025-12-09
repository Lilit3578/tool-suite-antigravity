//! Settings command module
//!
//! Handles application settings persistence.

use crate::shared::settings::AppSettings;
use crate::shared::error::AppResult;

/// Get current application settings (masked secrets)
#[tauri::command]
pub async fn get_settings() -> AppResult<AppSettings> {
    let settings = AppSettings::load().await.map_err(|e| crate::shared::error::AppError::Io(e))?;
    Ok(settings.masked())
}

/// Save application settings (handles secret merging)
#[tauri::command]
pub async fn save_settings(app_handle: tauri::AppHandle, mut settings: AppSettings) -> AppResult<()> {
    // Load current settings to retrieve existing secrets
    let current_settings = AppSettings::load().await.map_err(|e| crate::shared::error::AppError::Io(e))?;
    
    // Merge secrets: if new value is masked, keep the old value
    if settings.api_keys.translation_key == "********" {
        settings.api_keys.translation_key = current_settings.api_keys.translation_key;
    }
    if settings.api_keys.google_translate_api_key == "********" {
        settings.api_keys.google_translate_api_key = current_settings.api_keys.google_translate_api_key;
    }
    if settings.api_keys.currency_api_key == "********" {
        settings.api_keys.currency_api_key = current_settings.api_keys.currency_api_key;
    }

    settings.save(&app_handle).await.map_err(|e| crate::shared::error::AppError::Io(e))
}
