//! Command palette module (simplified)
//!
//! Handles command palette core logic: text capture, command item retrieval,
//! action execution, and usage tracking. Feature-specific logic is delegated
//! to the features module.

use crate::shared::types::*;
use crate::shared::error::{AppResult, AppError};
use crate::system::automation;
use crate::core::context;
use crate::core::features;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use serde_json;

/// Capture selected text from the active application
#[tauri::command]
pub async fn capture_selection(app: tauri::AppHandle, mode: Option<String>) -> AppResult<CaptureResult> {
    let source = mode.unwrap_or_else(|| "clipboard".to_string());
    
    println!("🔵 [DEBUG] [CaptureSelection] ========== CAPTURE SELECTION CALLED ==========");
    println!("🔵 [DEBUG] [CaptureSelection] Mode: {}", source);
    
    if source == "selection" {
        // Check active app before any operations
        let before_app = automation::get_active_app().ok();
        println!("🔵 [DEBUG] [CaptureSelection] Active app BEFORE operations: {:?}", before_app);
        
        // CRITICAL: Check if palette window is currently visible/focused
        let palette_is_open = app.get_webview_window("palette-window")
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(false);
        
        println!("🔵 [DEBUG] [CaptureSelection] Palette window open: {}", palette_is_open);
        
        if !palette_is_open {
            println!("🔵 [DEBUG] [CaptureSelection] Palette not open, restoring focus to previous app");
            // Only restore focus if palette is not open (e.g., widget is opening)
            let active_app = automation::get_active_app().ok();
            let target_app = active_app.clone().unwrap_or_else(|| "Finder".to_string());
            
            println!("🔵 [DEBUG] [CaptureSelection] Restoring focus to: {}", target_app);
            if let Err(e) = automation::restore_focus(&target_app) {
                eprintln!("🔴 [DEBUG] [CaptureSelection] ✗ Failed to restore focus to {}: {}", target_app, e);
            } else {
                println!("🔵 [DEBUG] [CaptureSelection] ✓ Successfully restored focus to {}", target_app);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            let after_restore_app = automation::get_active_app().ok();
            println!("🔵 [DEBUG] [CaptureSelection] Active app AFTER restore_focus: {:?}", after_restore_app);
        } else {
            println!("🔵 [DEBUG] [CaptureSelection] ✓ Palette is open, SKIPPING focus restore to prevent stealing focus");
        }
        
        // Simulate Cmd+C to capture selection
        println!("🔵 [DEBUG] [CaptureSelection] Simulating Cmd+C...");
        
        // Set ignore flag to prevent ghost copy in history
        let clipboard_state = app.state::<crate::core::clipboard::ClipboardState>();
        clipboard_state.ignore_next.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("🔵 [DEBUG] [CaptureSelection] 🚩 Ignore flag set before manual copy");

        if let Err(e) = automation::simulate_cmd_c() {
            eprintln!("🔴 [DEBUG] [CaptureSelection] ✗ Failed to simulate Cmd+C: {}", e);
        } else {
            println!("🔵 [DEBUG] [CaptureSelection] ✓ Successfully simulated Cmd+C");
        }
        
        // Check active app after Cmd+C
        let after_cmd_c_app = automation::get_active_app().ok();
        println!("🔵 [DEBUG] [CaptureSelection] Active app AFTER simulate_cmd_c: {:?}", after_cmd_c_app);
        if palette_is_open && after_cmd_c_app != before_app {
            eprintln!("🔴 [DEBUG] [CaptureSelection] ⚠️  FOCUS STOLEN by simulate_cmd_c! Changed from {:?} to {:?}", before_app, after_cmd_c_app);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        match app.clipboard().read_text() {
            Ok(text) => {
                println!("🔵 [DEBUG] [CaptureSelection] ✓ Successfully read {} bytes from clipboard", text.len());
                
                // NOTE: We do NOT refocus the palette window after capture.
                // Non-activating panels should NOT be focused as it activates the app.
                // The window will remain visible and interactable without becoming key.
                
                Ok(CaptureResult {
                    text,
                    source: automation::get_active_app().ok().unwrap_or_else(|| "unknown".to_string()),
                })
            }
            Err(e) => {
                eprintln!("🔴 [DEBUG] [CaptureSelection] ✗ Failed to read clipboard: {}", e);
                // NOTE: We do NOT refocus the palette window.
                // Non-activating panels should NOT be focused as it activates the app.
                Ok(CaptureResult {
                    text: String::new(),
                    source: "selection".to_string(),
                })
            }
        }
    } else {
        match app.clipboard().read_text() {
            Ok(text) => {
                Ok(CaptureResult {
                    text,
                    source: "clipboard".to_string(),
                })
            }
            Err(e) => {
                eprintln!("Failed to read clipboard: {}", e);
                Ok(CaptureResult {
                    text: String::new(),
                    source: "clipboard".to_string(),
                })
            }
        }
    }
}

/// Get the complete command index with usage weights
/// 
/// This returns ALL commands with their usage weights for frontend-side search.
/// The frontend handles fuzzy search/filtering using cmdk or Fuse.js.
/// This eliminates N+1 IPC queries during search.
#[tauri::command]
pub async fn get_command_index(
    _app: tauri::AppHandle,
    metrics: tauri::State<'_, context::UsageMetrics>,
) -> AppResult<Vec<CommandItem>> {
    // Get all command items from features
    let items = features::get_all_command_items();
    
    // Apply usage-based ranking (no context boost - frontend handles that)
    let ranked_items = context::rank_commands(
        items,
        |cmd| cmd.id.clone(),
        &metrics,
        None, // No context boost - frontend handles search
    );
    
    Ok(ranked_items)
}

/// Get command items (backward compatibility - calls get_command_index)
/// 
/// This maintains compatibility with existing frontend code.
/// Returns format: { commands: Vec<CommandItem>, detected_context?: ContextCategory }
#[tauri::command]
pub async fn get_command_items(
    _app: tauri::AppHandle,
    metrics: tauri::State<'_, context::UsageMetrics>,
    _captured_text: Option<String>,
) -> AppResult<serde_json::Value> {
    println!("[get_command_items] Called - getting all commands");
    // Just call get_command_index - frontend handles filtering
    let items = get_command_index(_app, metrics).await?;
    
    println!("[get_command_items] ✅ Returning {} commands", items.len());
    for (i, item) in items.iter().take(10).enumerate() {
        println!("[get_command_items]   {}. {} ({})", i + 1, item.label, item.id);
    }
    
    // Return in format expected by frontend: { commands: [...], detected_context?: ... }
    Ok(serde_json::json!({
        "commands": items
    }))
}

/// Execute an action
#[tauri::command]
pub async fn execute_action(request: ExecuteActionRequest) -> AppResult<ExecuteActionResponse> {
    println!("🔵 [execute_action] Received action: {:?}", request.action_type);
    println!("🔵 [execute_action] Params: {:?}", request.params);
    
    // features::execute_feature_action currently returns Result<..., String>.
    // We need to map it.
    features::execute_feature_action(&request).await.map_err(AppError::from)
}

/// Record command usage for intelligent ranking
#[tauri::command]
pub fn record_command_usage(
    metrics: tauri::State<context::UsageMetrics>,
    command_id: String,
) -> AppResult<()> {
    metrics.record_usage(&command_id);
    Ok(())
}
