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
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
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

    execute_applescript(&script)?;
    
    // Wait a bit for the app to come to front
    thread::sleep(Duration::from_millis(100));
    
    Ok(())
}

/// Simulate Cmd+C (copy) keystroke
/// Requires accessibility permissions
pub fn simulate_cmd_c() -> Result<(), String> {
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    let script = r#"
        tell application "System Events"
            keystroke "c" using command down
        end tell
    "#;

    execute_applescript(script)?;
    
    // Wait for clipboard to update
    thread::sleep(Duration::from_millis(100));
    
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
