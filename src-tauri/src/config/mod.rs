//! Window configuration registry
//!
//! Centralized window configuration to eliminate hardcoded dimensions.
//! Maps feature/widget IDs to WindowConfig structs.

use serde::{Deserialize, Serialize};

/// Window configuration for a feature/widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f64,
    pub height: f64,
    pub title: String,
    pub transparent: bool,
    pub resizable: bool,
}

impl WindowConfig {
    pub fn new(width: f64, height: f64, title: impl Into<String>) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            transparent: true,
            resizable: false,
        }
    }
}

/// Window registry mapping feature/widget IDs to configurations
pub fn get_window_config(feature_id: &str) -> WindowConfig {
    match feature_id {
        "palette" => WindowConfig::new(550.0, 328.0, "Command Palette"),
        "translator" => WindowConfig::new(700.0, 550.0, "Translator"),
        "currency" => WindowConfig::new(500.0, 400.0, "Currency Converter"),
        "unit_converter" => WindowConfig::new(500.0, 400.0, "Unit Converter"),
        "time_converter" => WindowConfig::new(600.0, 500.0, "Time Converter"),
        "definition" => WindowConfig::new(400.0, 500.0, "Definition"),
        "text_analyser" => WindowConfig::new(600.0, 450.0, "Text Analyser"),
        "settings" => WindowConfig::new(800.0, 600.0, "Settings"),
        "clipboard" => WindowConfig::new(400.0, 300.0, "Clipboard History"),
        _ => WindowConfig::new(500.0, 400.0, "Widget"), // Default fallback
    }
}

/// Get all window configurations (for debugging/admin)
pub fn get_all_configs() -> Vec<(&'static str, WindowConfig)> {
    vec![
        ("palette", get_window_config("palette")),
        ("translator", get_window_config("translator")),
        ("currency", get_window_config("currency")),
        ("unit_converter", get_window_config("unit_converter")),
        ("time_converter", get_window_config("time_converter")),
        ("definition", get_window_config("definition")),
        ("text_analyser", get_window_config("text_analyser")),
        ("settings", get_window_config("settings")),
        ("clipboard", get_window_config("clipboard")),
    ]
}
