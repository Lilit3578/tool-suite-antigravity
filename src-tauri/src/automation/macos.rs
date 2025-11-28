use std::process::Command;
use std::thread;
use std::time::Duration;

/// Execute AppleScript and return the output
fn execute_applescript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("AppleScript error: {}", stderr))
    }
}

/// Get the name of the currently active application
pub fn get_active_app() -> Result<String, String> {
    let script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            return frontApp
        end tell
    "#;

    execute_applescript(script)
}

/// Restore focus to a specific application by name
pub fn restore_focus(app_name: &str) -> Result<(), String> {
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
pub fn simulate_cmd_c() -> Result<(), String> {
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
pub fn simulate_cmd_v() -> Result<(), String> {
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
pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> Result<(), String> {
    // Restore focus to the original app
    restore_focus(app_name)?;
    
    // Wait the specified delay (80-150ms as per Electron spec)
    thread::sleep(Duration::from_millis(delay_ms));
    
    // Simulate Cmd+V
    simulate_cmd_v()?;
    
    println!("[AutoPaste] Completed paste flow to app: {}", app_name);
    Ok(())
}

/// Check if the app has accessibility permissions
pub fn check_accessibility_permissions() -> bool {
    // Try to get the active app - this requires accessibility permissions
    match get_active_app() {
        Ok(_) => true,
        Err(e) => {
            eprintln!("[Accessibility] Permission check failed: {}", e);
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
