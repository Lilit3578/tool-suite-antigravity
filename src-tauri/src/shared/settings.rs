use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkeys: HotkeySettings,
    pub api_keys: ApiKeys,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub command_palette: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
    pub translation_provider: String,
    pub translation_key: String,
    pub google_translate_api_key: String,
    pub currency_api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl AppSettings {
    pub fn get_settings_path() -> Result<PathBuf, String> {
        ProjectDirs::from("com", "antigravity", "productivity-widgets")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .ok_or_else(|| "Failed to determine config directory".to_string())
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::get_settings_path()?;
        
        if !path.exists() {
            let settings = Self::default();
            settings.save()?;
            return Ok(settings);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_settings_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))
    }
}
