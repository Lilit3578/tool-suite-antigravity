use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

/// Circuit breaker for paste operations
/// Tracks consecutive failures to prevent infinite retry loops
static PASTE_FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
const MAX_CONSECUTIVE_PASTE_FAILURES: u32 = 5;

/// Execute AppleScript and return the output
/// Includes comprehensive error logging
fn execute_applescript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| {
            let error_msg = format!("Failed to execute osascript: {}", e);
            eprintln!("[AppleScript] Execution failed: {}", error_msg);
            error_msg
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("AppleScript error: {}", stderr);
        eprintln!("[AppleScript] Script execution failed: {}", error_msg);
        eprintln!("[AppleScript] Exit code: {:?}", output.status.code());
        Err(error_msg)
    }
}

/// Get the name of the currently active application
/// Requires accessibility permissions
pub fn get_active_app() -> Result<String, String> {
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            return frontApp
        end tell
    "#;

    execute_applescript(script)
}

/// Restore focus to a specific application by name
/// Requires accessibility permissions
pub fn restore_focus(app_name: &str) -> Result<(), String> {
    println!("🔵 [DEBUG] [restore_focus] ========== RESTORE FOCUS CALLED ==========");
    println!("🔵 [DEBUG] [restore_focus] Target app: {}", app_name);
    
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    // Check active app before restore
    let before_app = get_active_app().ok();
    println!("🔵 [DEBUG] [restore_focus] Active app BEFORE restore: {:?}", before_app);
    
    // Check if app is running first
    let check_script = format!(
        r#"
        tell application "System Events"
            set appList to name of every process
            if appList contains "{}" then
                return "running"
            else
                return "not_running"
            end if
        end tell
        "#,
        app_name
    );
    
    match execute_applescript(&check_script) {
        Ok(status) if status.trim() == "running" => {
            // App is running, proceed
            println!("[RestoreFocus] Application '{}' is running, proceeding with focus restore", app_name);
        }
        Ok(_) => {
            let error_msg = format!("Application '{}' is not running", app_name);
            eprintln!("[RestoreFocus] {}", error_msg);
            return Err(error_msg);
        }
        Err(e) => {
            let error_msg = format!("Failed to check if app '{}' is running: {}", app_name, e);
            eprintln!("[RestoreFocus] {}", error_msg);
            return Err(error_msg);
        }
    }
    
    let script = format!(
        r#"
        tell application "{}"
            activate
        end tell
        "#,
        app_name
    );

    println!("🔵 [DEBUG] [restore_focus] Executing AppleScript to activate app...");
    match execute_applescript(&script) {
        Ok(_) => {
            println!("🔵 [DEBUG] [restore_focus] ✓ AppleScript executed successfully");
        }
        Err(e) => {
            eprintln!("🔴 [DEBUG] [restore_focus] ✗ AppleScript failed: {}", e);
            return Err(e);
        }
    }
    
    // Wait a bit for the app to come to front
    thread::sleep(Duration::from_millis(100));
    
    // Check active app after restore
    let after_app = get_active_app().ok();
    println!("🔵 [DEBUG] [restore_focus] Active app AFTER restore: {:?}", after_app);
    if after_app.as_ref() != Some(&app_name.to_string()) {
        eprintln!("🔴 [DEBUG] [restore_focus] ⚠️  Focus may not have been restored correctly. Expected: {}, Got: {:?}", app_name, after_app);
    } else {
        println!("🔵 [DEBUG] [restore_focus] ✓ Focus successfully restored to {}", app_name);
    }
    
    println!("🔵 [DEBUG] [restore_focus] ========== RESTORE FOCUS COMPLETE ==========");
    Ok(())
}

/// Simulate Cmd+C (copy) keystroke
/// Requires accessibility permissions
pub fn simulate_cmd_c() -> Result<(), String> {
    println!("🔵 [DEBUG] [simulate_cmd_c] ========== SIMULATE CMD+C CALLED ==========");
    
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    // Check active app before Cmd+C
    let before_app = get_active_app().ok();
    println!("🔵 [DEBUG] [simulate_cmd_c] Active app BEFORE Cmd+C: {:?}", before_app);
    
    let script = r#"
        tell application "System Events"
            keystroke "c" using command down
        end tell
    "#;

    println!("🔵 [DEBUG] [simulate_cmd_c] Executing AppleScript to send Cmd+C...");
    match execute_applescript(script) {
        Ok(_) => {
            println!("🔵 [DEBUG] [simulate_cmd_c] ✓ AppleScript executed successfully");
        }
        Err(e) => {
            eprintln!("🔴 [DEBUG] [simulate_cmd_c] ✗ AppleScript failed: {}", e);
            return Err(e);
        }
    }
    
    // Wait for clipboard to update
    thread::sleep(Duration::from_millis(100));
    
    // Check active app after Cmd+C
    let after_app = get_active_app().ok();
    println!("🔵 [DEBUG] [simulate_cmd_c] Active app AFTER Cmd+C: {:?}", after_app);
    if before_app != after_app {
        eprintln!("🔴 [DEBUG] [simulate_cmd_c] ⚠️  FOCUS CHANGED! From {:?} to {:?}", before_app, after_app);
    }
    
    println!("🔵 [DEBUG] [simulate_cmd_c] ========== SIMULATE CMD+C COMPLETE ==========");
    Ok(())
}

/// Simulate Cmd+V (paste) keystroke
/// Requires accessibility permissions
pub fn simulate_cmd_v() -> Result<(), String> {
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    let script = r#"
        tell application "System Events"
            keystroke "v" using command down
        end tell
    "#;

    execute_applescript(script)?;
    Ok(())
}

/// Simulate Cmd+C, wait, then restore focus and simulate Cmd+V
/// This is the full auto-paste flow
/// Includes circuit breaker to prevent infinite retry loops
pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> Result<(), String> {
    // Check circuit breaker
    let failures = PASTE_FAILURE_COUNT.load(Ordering::Relaxed);
    if failures >= MAX_CONSECUTIVE_PASTE_FAILURES {
        let error_msg = format!(
            "Circuit breaker: Too many consecutive paste failures ({}). Please check accessibility permissions in System Settings > Privacy & Security > Accessibility.",
            failures
        );
        eprintln!("[AutoPaste] {}", error_msg);
        return Err(error_msg);
    }
    
    // Check permissions before attempting operations
    if !check_accessibility_permissions() {
        PASTE_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        let error_msg = "Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string();
        eprintln!("[AutoPaste] {}", error_msg);
        return Err(error_msg);
    }
    
    // Restore focus to the original app with error handling
    if let Err(e) = restore_focus(app_name) {
        PASTE_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        let error_msg = format!("Failed to restore focus to '{}': {}", app_name, e);
        eprintln!("[AutoPaste] {}", error_msg);
        return Err(error_msg);
    }
    
    // Wait the specified delay (80-150ms as per Electron spec)
    thread::sleep(Duration::from_millis(delay_ms));
    
    // Simulate Cmd+V with error handling
    if let Err(e) = simulate_cmd_v() {
        PASTE_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        let error_msg = format!("Failed to paste: {}", e);
        eprintln!("[AutoPaste] {}", error_msg);
        return Err(error_msg);
    }
    
    // Reset failure count on success
    PASTE_FAILURE_COUNT.store(0, Ordering::Relaxed);
    
    println!("[AutoPaste] Completed paste flow to app: {}", app_name);
    Ok(())
}

/// Check if the app has accessibility permissions
/// This function directly tests permissions without calling other functions
/// to avoid circular dependencies
/// Returns true if permissions are granted, false otherwise
pub fn check_accessibility_permissions() -> bool {
    // Try to execute a simple System Events command that requires accessibility permissions
    let test_script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            return frontApp
        end tell
    "#;
    
    match execute_applescript(test_script) {
        Ok(_) => {
            println!("[Accessibility] Permissions check: Granted");
            true
        }
        Err(e) => {
            // Check for specific permission denial messages
            if e.contains("not allowed assistive") || e.contains("not allowed automation") {
                eprintln!("[Accessibility] ⚠️  Permissions denied. Please enable in System Settings > Privacy & Security > Accessibility");
            } else {
                eprintln!("[Accessibility] Permission check failed: {}", e);
            }
            false
        }
    }
}

/// Smart Command Palette opener with selection detection
/// 
/// This function implements clipboard diff-based selection detection:
/// 1. Snapshot clipboard (State A) - using SYSTEM clipboard, not Tauri API
/// 2. Simulate Cmd+C
/// 3. Wait for clipboard update
/// 4. Read clipboard again (State B) - using SYSTEM clipboard, not Tauri API
/// 5. Compare: if State B != State A, text was selected
/// 6. Position window accordingly (cursor vs center)
///
/// CRITICAL: Uses cli-clipboard instead of Tauri's clipboard API to avoid app activation
///
/// Returns: (has_selection: bool, selected_text: Option<String>)
pub async fn detect_text_selection(_app: &tauri::AppHandle) -> Result<(bool, Option<String>), String> {
    use cli_clipboard::{ClipboardContext, ClipboardProvider};
    
    println!("🔵 [DEBUG] [detect_text_selection] ========== DETECT TEXT SELECTION STARTED ==========");
    
    // Check active app before any operations
    let before_app = get_active_app().ok();
    println!("🔵 [DEBUG] [detect_text_selection] Active app BEFORE operations: {:?}", before_app);
    
    // STEP 1: Snapshot clipboard (State A) using SYSTEM clipboard
    println!("🔵 [DEBUG] [detect_text_selection] STEP 1: Snapshotting clipboard (State A) using SYSTEM API...");
    let state_a = match ClipboardContext::new()
        .and_then(|mut ctx| ctx.get_contents()) {
        Ok(text) => {
            println!("🔵 [DEBUG] [detect_text_selection] ✓ State A captured: {} bytes", text.len());
            text
        }
        Err(e) => {
            eprintln!("🔴 [DEBUG] [detect_text_selection] ✗ Failed to read system clipboard: {}", e);
            String::new() // Empty if clipboard read fails
        }
    };
    
    // Safe string truncation that respects UTF-8 character boundaries
    let state_a_preview = if state_a.len() > 50 {
        state_a.char_indices()
            .nth(50)
            .map(|(idx, _)| format!("{}...", &state_a[..idx]))
            .unwrap_or_else(|| state_a.clone())
    } else {
        state_a.clone()
    };
    println!("🔵 [DEBUG] [detect_text_selection] State A preview: {:?}", state_a_preview);
    
    // STEP 2: Trigger Cmd+C
    println!("🔵 [DEBUG] [detect_text_selection] STEP 2: Calling simulate_cmd_c()...");
    
    if let Err(e) = simulate_cmd_c() {
        eprintln!("🔴 [DEBUG] [detect_text_selection] ✗ Failed to simulate Cmd+C: {}", e);
        return Err(e);
    }
    
    // Check active app after Cmd+C
    let after_cmd_c_app = get_active_app().ok();
    println!("🔵 [DEBUG] [detect_text_selection] Active app AFTER simulate_cmd_c: {:?}", after_cmd_c_app);
    if before_app != after_cmd_c_app {
        eprintln!("🔴 [DEBUG] [detect_text_selection] ⚠️  FOCUS CHANGED after simulate_cmd_c! From {:?} to {:?}", before_app, after_cmd_c_app);
    }
    
    // STEP 3: Wait for clipboard to update (non-blocking)
    println!("🔵 [DEBUG] [detect_text_selection] STEP 3: Waiting 150ms for clipboard update...");
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // STEP 4: Read clipboard again (State B) using SYSTEM clipboard
    println!("🔵 [DEBUG] [detect_text_selection] STEP 4: Reading clipboard again (State B) using SYSTEM API...");
    let state_b = match ClipboardContext::new()
        .and_then(|mut ctx| ctx.get_contents()) {
        Ok(text) => {
            println!("🔵 [DEBUG] [detect_text_selection] ✓ State B captured: {} bytes", text.len());
            text
        }
        Err(e) => {
            eprintln!("🔴 [DEBUG] [detect_text_selection] ✗ Failed to read system clipboard: {}", e);
            String::new()
        }
    };
    
    // Safe string truncation that respects UTF-8 character boundaries
    let state_b_preview = if state_b.len() > 50 {
        state_b.char_indices()
            .nth(50)
            .map(|(idx, _)| format!("{}...", &state_b[..idx]))
            .unwrap_or_else(|| state_b.clone())
    } else {
        state_b.clone()
    };
    println!("🔵 [DEBUG] [detect_text_selection] State B preview: {:?}", state_b_preview);
    
    // STEP 5: Compare states (The Diff Check)
    println!("🔵 [DEBUG] [detect_text_selection] STEP 5: Comparing states...");
    let has_selection = state_b != state_a && !state_b.trim().is_empty();
    
    if has_selection {
        println!("🔵 [DEBUG] [detect_text_selection] ✓ Text was selected (diff detected)");
    } else {
        println!("🔵 [DEBUG] [detect_text_selection] ✗ No text selected (no diff or empty)");
    }
    
    // Final active app check
    let final_app = get_active_app().ok();
    println!("🔵 [DEBUG] [detect_text_selection] Final active app: {:?}", final_app);
    
    // CRITICAL CHECK: Did focus change during detection?
    if before_app.is_some() && final_app != before_app {
        eprintln!("🔴 [DEBUG] [detect_text_selection] ⚠️  FOCUS WAS STOLEN! Started with {:?}, ended with {:?}", before_app, final_app);
    } else {
        println!("🔵 [DEBUG] [detect_text_selection] ✓ Focus remained stable: {:?}", before_app);
    }
    
    println!("🔵 [DEBUG] [detect_text_selection] ========== DETECT TEXT SELECTION COMPLETE ==========");
    
    Ok((has_selection, if has_selection { Some(state_b) } else { None }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run manually as it requires accessibility permissions
    fn test_get_active_app() {
        match get_active_app() {
            Ok(app) => {
                println!("Active app: {}", app);
                assert!(!app.is_empty());
            }
            Err(e) => {
                println!("Error (may need accessibility permissions): {}", e);
            }
        }
    }

    #[test]
    #[ignore] // Only run manually as it affects the system
    fn test_simulate_cmd_c() {
        // This test would actually trigger Cmd+C on the system
        // Only run manually in a controlled environment
        match simulate_cmd_c() {
            Ok(_) => println!("Cmd+C simulated successfully"),
            Err(e) => println!("Error: {}", e),
        }
    }
}
