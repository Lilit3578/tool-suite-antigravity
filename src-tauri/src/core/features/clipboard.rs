//! Clipboard history feature
//!
//! Provides clipboard history management and paste automation.

use crate::shared::types::*;
use crate::core::clipboard::{ClipboardHistory, ClipboardItem, ClipboardMonitor};
use crate::system::automation;
use super::Feature;
use tauri::Manager;
use std::sync::{Arc, Mutex};

pub struct ClipboardFeature;

impl Feature for ClipboardFeature {
    fn id(&self) -> &str {
        "clipboard"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_clipboard".to_string(),
            label: "Clipboard History".to_string(),
            description: Some("View and paste from clipboard history".to_string()),
            action_type: None,
            widget_type: Some("clipboard".to_string()),
            category: None, // Will be assigned in get_all_command_items()
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // Clipboard doesn't have direct actions (uses widget)
        vec![]
    }
    
    fn execute_action(
        &self,
        _action: &ActionType,
        _params: &serde_json::Value,
    ) -> Result<ExecuteActionResponse, String> {
        Err("Clipboard feature doesn't support direct actions".to_string())
    }
}

/// Get clipboard history items
#[tauri::command]
pub fn get_clipboard_history(history: tauri::State<ClipboardHistory>) -> Result<Vec<ClipboardItem>, String> {
    Ok(history.get_items())
}

/// Paste a clipboard item to the active application
#[tauri::command]
pub async fn paste_clipboard_item(
    app: tauri::AppHandle,
    history: tauri::State<'_, ClipboardHistory>,
    last_active_app: tauri::State<'_, Arc<Mutex<Option<String>>>>,
    item_id: String,
) -> Result<(), String> {
    let item = history
        .get_item_by_id(&item_id)
        .ok_or_else(|| "Clipboard item not found".to_string())?;

    println!("[PasteItem] Pasting item: {}", item.id);

    history.set_skip_next_add(true);

    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(item.content.clone())
        .map_err(|e| format!("Failed to write to clipboard: {}", e))?;

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
        window.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
    }
    if let Some(window) = app.get_webview_window("clipboard-window") {
        window.hide().map_err(|e| format!("Failed to hide clipboard: {}", e))?;
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
pub fn clear_clipboard_history(history: tauri::State<ClipboardHistory>) -> Result<(), String> {
    history.clear();
    Ok(())
}

/// Toggle clipboard monitoring on/off
#[tauri::command]
pub fn toggle_clipboard_monitor(monitor: tauri::State<ClipboardMonitor>) -> Result<bool, String> {
    let enabled = monitor.toggle();
    Ok(enabled)
}

/// Get clipboard monitor status
#[tauri::command]
pub fn get_clipboard_monitor_status(monitor: tauri::State<ClipboardMonitor>) -> Result<bool, String> {
    Ok(monitor.is_enabled())
}
