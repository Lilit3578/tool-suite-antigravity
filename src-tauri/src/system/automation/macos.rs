use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSString, NSUInteger};
use objc::{msg_send, sel, sel_impl, class};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventFlags, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Circuit breaker for paste operations
/// Tracks consecutive failures to prevent infinite retry loops
static PASTE_FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
const MAX_CONSECUTIVE_PASTE_FAILURES: u32 = 5;

// Key codes for macOS (ANSI standard)
const K_VK_ANSI_C: CGKeyCode = 0x08;
const K_VK_ANSI_V: CGKeyCode = 0x09;

/// Check if the app has accessibility permissions
/// Uses native Accessibility API (AXIsProcessTrusted)
pub fn check_accessibility_permissions() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

/// Get the name of the currently active application
/// Uses NSWorkspace via Cocoa/ObjC
pub fn get_active_app() -> Result<String, String> {
    if !check_accessibility_permissions() {
         return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility.".to_string());
    }
    
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        
        if front_app == nil {
            return Err("No frontmost application found".to_string());
        }
        
        let name: id = msg_send![front_app, localizedName];
        if name == nil {
            return Ok("Unknown".to_string());
        }
        
        let name_cstr = std::ffi::CStr::from_ptr(NSString::UTF8String(name));
        Ok(name_cstr.to_string_lossy().into_owned())
    }
}

/// Restore focus to a specific application by name
/// Iterates running applications to find match and activates it
pub fn restore_focus(app_name: &str) -> Result<(), String> {
    println!("🔵 [DEBUG] [restore_focus] ========== RESTORE FOCUS CALLED ==========");
    println!("🔵 [DEBUG] [restore_focus] Target app: {}", app_name);
    
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted.".to_string());
    }

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let running_apps: id = msg_send![workspace, runningApplications];
        let count: NSUInteger = msg_send![running_apps, count];
        
        let mut target_app: id = nil;
        
        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex:i];
            let name: id = msg_send![app, localizedName];
            
            if name != nil {
                let name_cstr = std::ffi::CStr::from_ptr(NSString::UTF8String(name));
                let name_str = name_cstr.to_string_lossy();
                
                if name_str == app_name {
                    target_app = app;
                    break;
                }
            }
        }
        
        if target_app != nil {
            println!("[RestoreFocus] Found app '{}', activating...", app_name);
            // NSApplicationActivateIgnoringOtherApps = 1 << 0
            let options: NSUInteger = 1; 
            let _: bool = msg_send![target_app, activateWithOptions:options];
            
            // Wait a bit for focus to settle
            thread::sleep(Duration::from_millis(100));
            
            println!("🔵 [DEBUG] [restore_focus] ✓ Focus restored");
            Ok(())
        } else {
            let msg = format!("Application '{}' not found running", app_name);
            eprintln!("[RestoreFocus] {}", msg);
            Err(msg)
        }
    }
}

/// Helper to simulate a keystroke with modifiers
fn simulate_keypress(key_code: CGKeyCode, flags: CGEventFlags) -> Result<(), String> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create CGEventSource".to_string())?;
    
    // Key Down
    let key_down = CGEvent::new_keyboard_event(source.clone(), key_code, true)
        .map_err(|_| "Failed to create key down event".to_string())?;
    key_down.set_flags(flags);
    key_down.post(CGEventTapLocation::HID);
    
    // Key Up
    let key_up = CGEvent::new_keyboard_event(source, key_code, false)
        .map_err(|_| "Failed to create key up event".to_string())?;
    key_up.set_flags(flags);
    key_up.post(CGEventTapLocation::HID);
    
    Ok(())
}

/// Simulate Cmd+C (copy) using Core Graphics
pub fn simulate_cmd_c() -> Result<(), String> {
    println!("🔵 [DEBUG] [simulate_cmd_c] (Native) Triggering Cmd+C...");
    
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted.".to_string());
    }
    
    simulate_keypress(K_VK_ANSI_C, CGEventFlags::CGEventFlagCommand)?;
    
    // Wait for clipboard update naturally
    thread::sleep(Duration::from_millis(100));
    
    println!("🔵 [DEBUG] [simulate_cmd_c] (Native) Complete");
    Ok(())
}

/// Simulate Cmd+V (paste) using Core Graphics
pub fn simulate_cmd_v() -> Result<(), String> {
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted.".to_string());
    }
    
    simulate_keypress(K_VK_ANSI_V, CGEventFlags::CGEventFlagCommand)?;
    Ok(())
}

/// Auto-paste flow: Restore focus -> Wait -> Paste
pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> Result<(), String> {
    // Check circuit breaker
    let failures = PASTE_FAILURE_COUNT.load(Ordering::Relaxed);
    if failures >= MAX_CONSECUTIVE_PASTE_FAILURES {
        return Err(format!("Circuit breaker: Too many consecutive paste failures ({})", failures));
    }
    
    if let Err(e) = restore_focus(app_name) {
        PASTE_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }
    
    thread::sleep(Duration::from_millis(delay_ms));
    
    if let Err(e) = simulate_cmd_v() {
        PASTE_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(e);
    }
    
    // Reset on success
    PASTE_FAILURE_COUNT.store(0, Ordering::Relaxed);
    println!("[AutoPaste] (Native) Completed paste flow to {}", app_name);
    Ok(())
}

/// Smart selection detection (same logic, using new native helpers)
pub async fn detect_text_selection(_app: &tauri::AppHandle) -> Result<(bool, Option<String>), String> {
    use cli_clipboard::{ClipboardContext, ClipboardProvider};
    
    let state_a = ClipboardContext::new()
        .and_then(|mut ctx| ctx.get_contents())
        .map_err(|e| e.to_string())?;
        
    simulate_cmd_c()?;
    
    thread::sleep(Duration::from_millis(150));
    
    let state_b = ClipboardContext::new()
        .and_then(|mut ctx| ctx.get_contents())
        .map_err(|e| e.to_string())?;
        
    let has_selection = state_b != state_a && !state_b.trim().is_empty();
    
    Ok((has_selection, if has_selection { Some(state_b) } else { None }))
}
