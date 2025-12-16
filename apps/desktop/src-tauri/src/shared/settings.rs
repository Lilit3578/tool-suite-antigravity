use serde::{Deserialize, Serialize};
use ts_rs::TS;
use tokio::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use tauri::AppHandle;
use crate::shared::events::AppEvent;
use crate::shared::emit::emit_event;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/settings.ts")]
pub struct AppSettings {
    pub hotkeys: HotkeySettings,
    pub api_keys: ApiKeys,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/settings.ts")]
pub struct HotkeySettings {
    pub command_palette: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/settings.ts")]
pub struct ApiKeys {
    pub translation_provider: String,
    pub translation_key: String,
    pub google_translate_api_key: String,
    pub currency_api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/settings.ts")]
pub struct UserPreferences {
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub default_currency_from: String,
    pub default_currency_to: String,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkeys: HotkeySettings {
                command_palette: "Control+Shift+L".to_string(),
            },
            api_keys: ApiKeys {
                translation_provider: "google".to_string(),
                translation_key: String::new(),
                google_translate_api_key: String::new(),
                currency_api_key: String::new(),
            },
            preferences: UserPreferences {
                default_source_lang: "auto".to_string(),
                default_target_lang: "en".to_string(),
                default_currency_from: "USD".to_string(),
                default_currency_to: "EUR".to_string(),
                theme: "system".to_string(),
            },
        }
    }
}

use keyring::Entry;

const KEYRING_SERVICE: &str = "productivity-widgets";

impl AppSettings {
    pub fn get_settings_path() -> Result<PathBuf, String> {
        ProjectDirs::from("com", "antigravity", "productivity-widgets")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .ok_or_else(|| "Failed to determine config directory".to_string())
    }

    pub async fn load() -> Result<Self, String> {
        let path = Self::get_settings_path()?;
        
        let mut settings = if !path.exists() {
            let s = Self::default();
            // Don't save default here, wait for explicit save
            s
        } else {
            let content = fs::read_to_string(&path).await
                .map_err(|e| format!("Failed to read settings file: {}", e))?;
            
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse settings: {}", e))?
        };

        // Load secrets from keyring
        settings.load_secrets_from_keyring().await?;
        
        Ok(settings)
    }

    /// Helper to save string to disk without emission (masks secrets first)
    async fn save_to_disk(&self) -> Result<(), String> {
        let path = Self::get_settings_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Create a copy for disk storage with secrets removed
        let mut disk_copy = self.clone();
        disk_copy.clear_secrets();

        let content = serde_json::to_string_pretty(&disk_copy)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        fs::write(&path, content).await
            .map_err(|e| format!("Failed to write settings file: {}", e))
    }

    /// Save settings (secrets to keyring, rest to disk) and emit update
    pub async fn save(&self, app: &AppHandle) -> Result<(), String> {
        // Save secrets to keyring first
        self.save_secrets_to_keyring()?;

        // Save stripped config to disk
        self.save_to_disk().await?;
            
        // Emit update event (send FULL settings to app state listeners, they might need the keys)
        // Wait, should we emit masked or full? 
        // Backend listeners need full keys. Frontend listeners will get this event.
        // If we emit full keys, frontend gets them.
        // We should probably emit MASKED keys to frontend, but backend components typically reload from disk/keyring or have their own state.
        // However, `emit_event` sends to frontend.
        // Let's emit MASKED settings for security.
        let masked = self.masked();
        emit_event(app, AppEvent::SettingsUpdated(masked));
        
        Ok(())
    }

    /// Return a copy of settings with secrets masked (for frontend/logging)
    pub fn masked(&self) -> Self {
        let mut copy = self.clone();
        if !copy.api_keys.translation_key.is_empty() {
            copy.api_keys.translation_key = "********".to_string();
        }
        if !copy.api_keys.google_translate_api_key.is_empty() {
            copy.api_keys.google_translate_api_key = "********".to_string();
        }
        if !copy.api_keys.currency_api_key.is_empty() {
            copy.api_keys.currency_api_key = "********".to_string();
        }
        copy
    }

    /// Clear secrets (for disk storage)
    fn clear_secrets(&mut self) {
        self.api_keys.translation_key = String::new();
        self.api_keys.google_translate_api_key = String::new();
        self.api_keys.currency_api_key = String::new();
    }

    async fn load_secrets_from_keyring(&mut self) -> Result<(), String> {
        let get_secret = |key: &str| -> Option<String> {
            match Entry::new(KEYRING_SERVICE, key) {
                Ok(entry) => match entry.get_password() {
                    Ok(pw) => Some(pw),
                    Err(keyring::Error::NoEntry) => None,
                    Err(e) => {
                        eprintln!("[Settings] Keyring error for {}: {}", key, e);
                        None
                    }
                },
                Err(e) => {
                    eprintln!("[Settings] Failed to access keyring for {}: {}", key, e);
                    None
                }
            }
        };

        if let Some(s) = get_secret("translation_key") { self.api_keys.translation_key = s; }
        if let Some(s) = get_secret("google_translate_api_key") { self.api_keys.google_translate_api_key = s; }
        if let Some(s) = get_secret("currency_api_key") { self.api_keys.currency_api_key = s; }

        Ok(())
    }

    fn save_secrets_to_keyring(&self) -> Result<(), String> {
        let set_secret = |key: &str, value: &str| -> Result<(), String> {
            if value.is_empty() || value == "********" {
                return Ok(()); // Don't save empty/masked
            }
            let entry = Entry::new(KEYRING_SERVICE, key)
                .map_err(|e| format!("Keyring init error: {}", e))?;
            entry.set_password(value)
                .map_err(|e| format!("Failed to save {} to keyring: {}", key, e))
        };

        set_secret("translation_key", &self.api_keys.translation_key)?;
        set_secret("google_translate_api_key", &self.api_keys.google_translate_api_key)?;
        set_secret("currency_api_key", &self.api_keys.currency_api_key)?;

        Ok(())
    }
}
