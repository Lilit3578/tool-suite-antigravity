//! Clipboard module
//!
//! Provides clipboard history tracking and monitoring functionality.
//!
//! This module contains two main components:
//! - `history`: Manages clipboard history with deduplication and capacity limits
//! - `monitor`: Background thread that monitors clipboard changes

pub mod history;
pub mod monitor;

pub use history::{ClipboardHistory, ClipboardItem};
pub use monitor::ClipboardMonitor;

/// Write text to clipboard using Tauri's clipboard manager
#[tauri::command]
pub fn write_clipboard_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}
