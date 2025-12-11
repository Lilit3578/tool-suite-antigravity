//! Clipboard history feature
//!
//! Provides clipboard history management and paste automation.

use crate::shared::types::*;
use crate::core::clipboard::{ClipboardHistory, ClipboardMonitor};

use crate::core::context;
use crate::system::automation;
use super::{FeatureSync, FeatureAsync};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use tauri::Manager;
use crate::core::context::category::ContextCategory;

#[derive(Clone)]
pub struct ClipboardFeature;

impl FeatureSync for ClipboardFeature {
    fn id(&self) -> &str {
        "clipboard"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        // Clipboard History is no longer a standalone widget
        // Backend tracking remains active, but no UI entry point
        vec![]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Removed: "Show Clipboard History", "Clear History", "Pause/Resume"
        // Backend commands still available via Tauri API for future use
        vec![]
    }
    
    fn get_context_boost(&self, _captured_text: &str) -> HashMap<String, f64> {
        HashMap::new()
    }
}

#[async_trait]
impl FeatureAsync for ClipboardFeature {
    async fn execute_action(
        &self,
        _action: &ActionType,
        _params: &serde_json::Value,
    ) -> crate::shared::error::AppResult<ExecuteActionResponse> {
        Err(crate::shared::error::AppError::Unknown("Unsupported action type".to_string()))
    }
}

/// Get clipboard history items
#[tauri::command]
pub fn get_clipboard_history(history: tauri::State<ClipboardHistory>) -> crate::shared::error::AppResult<Vec<ClipboardHistoryItem>> {
    history.get_items()
}

/// Paste a clipboard item to the active application
#[tauri::command]
pub async fn paste_clipboard_item(
    app: tauri::AppHandle,
    history: tauri::State<'_, ClipboardHistory>,
    last_active_app: tauri::State<'_, Arc<Mutex<Option<String>>>>,
    item_id: String,
) -> crate::shared::error::AppResult<()> {
    let item = history
        .get_item_by_id(&item_id)?
        .ok_or_else(|| crate::shared::error::AppError::Validation("Clipboard item not found".to_string()))?;

    println!("[PasteItem] Pasting item: {}", item.id);

    history.set_skip_next_add(true);

    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(item.content.clone())
        .map_err(|e| crate::shared::error::AppError::System(format!("Failed to write to clipboard: {}", e)))?;

    let target_app = {
        let last_app_guard = match last_active_app.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[PasteItem] Mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        if let Some(app_name) = last_app_guard.as_ref() {
            println!("[PasteItem] Using stored last active app: {}", app_name);
            app_name.clone()
        } else if let Some(source) = &item.source_app {
            println!("[PasteItem] Using item source app: {}", source);
            source.clone()
        } else {
            let fallback = automation::get_active_app().unwrap_or_else(|_| "Finder".to_string());
            println!("[PasteItem] Using fallback app: {}", fallback);
            fallback
        }
    };

    println!("[PasteItem] Target app: {}", target_app);

    if let Some(window) = app.get_webview_window("palette-window") {
        window.hide().map_err(|e| crate::shared::error::AppError::System(format!("Failed to hide palette: {}", e)))?;
    }
    if let Some(window) = app.get_webview_window("clipboard-window") {
        window.hide().map_err(|e| crate::shared::error::AppError::System(format!("Failed to hide clipboard: {}", e)))?;
    }

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match automation::auto_paste_flow(&target_app, 120) {
            Ok(_) => {
                println!("[PasteItem] ✅ Successfully pasted to application: {}", target_app);
            }
            Err(e) => {
                eprintln!("[PasteItem] ❌ Auto-paste failed: {}", e);
                eprintln!("[PasteItem] Target app: {}", target_app);
            }
        }
    });

    Ok(())
}

/// Clear all clipboard history
#[tauri::command]
pub fn clear_clipboard_history(history: tauri::State<ClipboardHistory>) -> crate::shared::error::AppResult<()> {
    history.clear()
}

/// Toggle clipboard monitoring on/off
#[tauri::command]
pub fn toggle_clipboard_monitor(monitor: tauri::State<ClipboardMonitor>) -> crate::shared::error::AppResult<bool> {
    let enabled = monitor.toggle();
    Ok(enabled)
}

/// Get clipboard monitor status
#[tauri::command]
pub fn get_clipboard_monitor_status(monitor: tauri::State<ClipboardMonitor>) -> crate::shared::error::AppResult<bool> {
    Ok(monitor.is_enabled())
}
