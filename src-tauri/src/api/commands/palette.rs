//! Command palette module (simplified)
//!
//! Handles command palette core logic: text capture, command item retrieval,
//! action execution, and usage tracking. Feature-specific logic is delegated
//! to the features module.

use crate::types::*;
use crate::automation;
use crate::context;
use crate::features;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Capture selected text from the active application
#[tauri::command]
pub async fn capture_selection(app: tauri::AppHandle, mode: Option<String>) -> Result<CaptureResult, String> {
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

/// Get all available command items with intelligent ranking
#[tauri::command]
pub async fn get_command_items(
    _app: tauri::AppHandle,
    metrics: tauri::State<'_, context::UsageMetrics>,
    captured_text: Option<String>,
) -> Result<Vec<CommandItem>, String> {
    // Get all command items from features
    let items = features::get_all_command_items();
    
    // Apply context-aware ranking if we have captured text
    let context_boost = if let Some(ref text) = captured_text {
        Some(features::get_context_boost(text))
    } else {
        None
    };
    
    // Rank commands using usage metrics and context
    let ranked_items = context::rank_commands(
        items,
        |cmd| cmd.id.clone(),
        &metrics,
        context_boost,
    );
    
    Ok(ranked_items)
}

/// Execute an action
#[tauri::command]
pub async fn execute_action(request: ExecuteActionRequest) -> Result<ExecuteActionResponse, String> {
    features::execute_feature_action(&request)
}

/// Record command usage for intelligent ranking
#[tauri::command]
pub fn record_command_usage(
    metrics: tauri::State<context::UsageMetrics>,
    command_id: String,
) -> Result<(), String> {
    metrics.record_usage(&command_id);
    Ok(())
}
