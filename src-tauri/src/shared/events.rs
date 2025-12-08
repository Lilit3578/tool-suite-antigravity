use serde::{Serialize, Deserialize};
use ts_rs::TS;
use super::types::ClipboardHistoryItem;
use super::settings::AppSettings;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "event", content = "payload")] // Tagged enum for easier frontend parsing
#[ts(export, export_to = "../../src/types/events.ts")] // Separate file for events
pub enum AppEvent {
    #[serde(rename = "clipboard://updated")]
    ClipboardUpdated(ClipboardHistoryItem),
    
    #[serde(rename = "settings://updated")]
    SettingsUpdated(AppSettings),
    
    #[serde(rename = "window://focus-changed")]
    WindowFocusChanged(bool),
}
