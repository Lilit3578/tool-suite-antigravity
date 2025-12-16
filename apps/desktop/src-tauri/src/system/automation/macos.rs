use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSString, NSUInteger, NSArray};
use objc::{msg_send, sel, sel_impl, class};
use objc::rc::autoreleasepool;
use objc::runtime::Class;
use std::ffi::CStr;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventFlags, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use crate::shared::error::{AppError, AppResult};

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

/// Ensure accessibility permissions are granted, prompting user if needed
/// 
/// This function will trigger the macOS system dialog if permissions are not granted.
/// Returns true if permissions are already granted, false if they need to be granted.
/// 
/// IMPORTANT: If this returns false, the user MUST:
/// 1. Grant permissions in System Settings > Privacy & Security > Accessibility
/// 2. Restart the application
pub fn ensure_accessibility_permissions() -> bool {
    use core_foundation::base::{CFRelease, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> bool;
    }
    
    unsafe {
        // Create the prompt option key
        let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::true_value();
        
        // Create options dictionary with prompt enabled
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        
        // Check permissions with prompt
        let is_trusted = AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
        
        if is_trusted {
            println!("✅ Accessibility permissions granted");
        } else {
            println!("⚠️  Accessibility permissions NOT granted - system prompt shown");
            println!("💡 User must grant permissions in System Settings and restart the app");
        }
        
        is_trusted
    }
}

/// Get the name of the currently active application
/// Uses NSWorkspace via Cocoa/ObjC
pub fn get_active_app() -> AppResult<String> {
    if !check_accessibility_permissions() {
         return Err(AppError::System("Accessibility permissions denied".to_string()));
    }
    
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        
        if front_app == nil {
            return Err(AppError::Io("No frontmost application found".to_string()));
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
pub fn restore_focus(app_name: &str) -> AppResult<()> {
    println!("🔵 [DEBUG] [restore_focus] ========== RESTORE FOCUS CALLED ==========");
    println!("🔵 [DEBUG] [restore_focus] Target app: {}", app_name);
    
    if !check_accessibility_permissions() {
        return Err(AppError::System("Accessibility permissions denied".to_string()));
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
            Err(AppError::Io(msg))
        }
    }
}

/// Helper to simulate a keystroke with modifiers
fn simulate_keypress(key_code: CGKeyCode, flags: CGEventFlags) -> AppResult<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| AppError::Io("Failed to create CGEventSource".to_string()))?;
    
    // Key Down
    let key_down = CGEvent::new_keyboard_event(source.clone(), key_code, true)
        .map_err(|_| AppError::Io("Failed to create key down event".to_string()))?;
    key_down.set_flags(flags);
    key_down.post(CGEventTapLocation::HID);
    
    // Key Up
    let key_up = CGEvent::new_keyboard_event(source, key_code, false)
        .map_err(|_| AppError::Io("Failed to create key up event".to_string()))?;
    key_up.set_flags(flags);
    key_up.post(CGEventTapLocation::HID);
    
    Ok(())
}

/// Simulate Cmd+C (copy) using Core Graphics
/// DEPRECATED: Use capture_selection() instead to avoid race conditions
pub fn simulate_cmd_c() -> AppResult<()> {
    println!("🔵 [DEBUG] [simulate_cmd_c] (Native) Triggering Cmd+C...");
    
    if !check_accessibility_permissions() {
        return Err(AppError::System("Accessibility permissions denied".to_string()));
    }
    
    simulate_keypress(K_VK_ANSI_C, CGEventFlags::CGEventFlagCommand)?;
    
    // Wait for clipboard update naturally
    thread::sleep(Duration::from_millis(100));
    
    println!("🔵 [DEBUG] [simulate_cmd_c] (Native) Complete");
    Ok(())
}

/// Simulate Cmd+V (paste) using Core Graphics
pub fn simulate_cmd_v() -> AppResult<()> {
    if !check_accessibility_permissions() {
        return Err(AppError::System("Accessibility permissions denied".to_string()));
    }
    
    simulate_keypress(K_VK_ANSI_V, CGEventFlags::CGEventFlagCommand)?;
    Ok(())
}

/// Auto-paste flow: Restore focus -> Wait -> Paste
pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> AppResult<()> {
    // Check circuit breaker
    let failures = PASTE_FAILURE_COUNT.load(Ordering::Relaxed);
    if failures >= MAX_CONSECUTIVE_PASTE_FAILURES {
        return Err(AppError::Io(format!("Circuit breaker: Too many consecutive paste failures ({})", failures)));
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

/// Put system to sleep using IOKit (native API, no shell commands)
/// 
/// SAFETY: This function uses unsafe blocks to call private macOS frameworks.
/// It directly interfaces with IOKit to send sleep commands to the system.
pub fn sleep_system() -> AppResult<()> {
    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMSleepSystem();
    }

    unsafe {
        IOPMSleepSystem();
    }
    Ok(())
}

/// Lock screen using ScreenSaver framework (native API, no shell commands)
/// 
/// SAFETY: This function uses unsafe blocks to call macOS ScreenSaver framework.
/// It directly requests the screen saver to start, which locks the screen.
pub fn lock_screen() -> AppResult<()> {
    unsafe {
        let screen_saver: id = msg_send![class!(NSScreenSaver), sharedScreenSaver];
        if screen_saver == nil {
            return Err(AppError::Io("Failed to get NSScreenSaver instance".to_string()));
        }
        
        let _: () = msg_send![screen_saver, lockScreen];
    }
    Ok(())
}

// Constants for deep-search selection strategy
const MAX_RECURSION_DEPTH: u8 = 3;
const MAX_CLIPBOARD_POLL_ATTEMPTS: u8 = 20;
const CLIPBOARD_POLL_INTERVAL_MS: u64 = 50;

/// Capture selected text using macOS Accessibility API with Deep-Search Selection Strategy
/// 
/// Strategy:
/// 1. Recursive AX Tree Walk: Searches nested UI elements (depth 3) to find selected text
///    - Works with Chrome/Electron where focused element is a container
/// 2. Transactional "Ghost" Copy Fallback: If AX fails, simulates Cmd+C and polls clipboard
///    - Waits for clipboard change count to increment before reading
///    - Optionally restores previous clipboard content
/// 
/// IMPORTANT: Always writes captured text to clipboard so frontend can read it
pub fn capture_selection(ignore_flag: Option<Arc<AtomicBool>>) -> AppResult<Option<String>> {
    // Use catch_unwind to prevent panics from crashing the app
    let result = std::panic::catch_unwind(|| {
        if !check_accessibility_permissions() {
            return Err(AppError::System("Accessibility permissions denied".to_string()));
        }

        // Step 1: Try Native AX API (Deep Search)
        println!("[CaptureSelection] 🔍 Attempting recursive AX tree walk...");
        match get_ax_selection_recursive() {
            Ok(Some(text)) => {
                println!("[CaptureSelection] ✅ Success via AX API (recursive search)");
                
                // CRITICAL FIX: Write to clipboard so frontend can read it
                // Set ignore flag because we are manually writing to clipboard
                if let Some(flag) = &ignore_flag {
                    flag.store(true, Ordering::SeqCst);
                    println!("[CaptureSelection] 🚩 Ignore flag set for manual write");
                }
                
                if let Err(e) = write_text_to_clipboard(&text) {
                    eprintln!("[CaptureSelection] ⚠️ Failed to write to clipboard: {:?}", e);
                }
                
                return Ok(Some(text));
            }
            Ok(None) => {
                println!("[CaptureSelection] ⚠️  AX search returned None, falling back...");
            }
            Err(e) => {
                println!("[CaptureSelection] ⚠️  AX search error: {:?}, falling back...", e);
            }
        }

        // Step 2: Fallback: Transactional "Ghost" Copy
        println!("[CaptureSelection] ⚠️  Falling back to simulated copy...");
        match capture_via_simulated_copy(ignore_flag) {
            Ok(Some(text)) => {
                println!("[CaptureSelection] ✅ Success via fallback (simulated copy)");
                // Clipboard is already updated by simulated copy
                Ok(Some(text))
            }
            Ok(None) => {
                println!("[CaptureSelection] ❌ No text captured via fallback");
                Ok(None)
            }
            Err(e) => {
                println!("[CaptureSelection] ❌ Fallback error: {:?}", e);
                Err(e)
            }
        }
    });

    match result {
        Ok(res) => res,
        Err(_) => {
            eprintln!("[CaptureSelection] ❌ PANIC caught in capture_selection!");
            Err(AppError::Io("Internal error: panic in capture_selection".to_string()))
        }
    }
}

/// Helper function to write text to clipboard
fn write_text_to_clipboard(text: &str) -> AppResult<()> {
    unsafe {
        let pb: id = msg_send![class!(NSPasteboard), generalPasteboard];
        if pb == nil {
            return Err(AppError::Io("Failed to get NSPasteboard".to_string()));
        }
        
        // Clear clipboard
        let _: () = msg_send![pb, clearContents];
        
        // Create NSString
        let ns_string = NSString::alloc(nil).init_str(text);
        if ns_string == nil {
            return Err(AppError::Io("Failed to create NSString".to_string()));
        }
        
        // Write to clipboard
        let array: id = msg_send![class!(NSArray), arrayWithObject:ns_string];
        let success: bool = msg_send![pb, writeObjects:array];
        
        if !success {
            return Err(AppError::Io("Failed to write to clipboard".to_string()));
        }
        
        println!("[WriteClipboard] ✅ Wrote {} bytes to clipboard", text.len());
        Ok(())
    }
}

/// Recursive AX tree walk to find selected text in nested UI elements
/// 
/// Searches the focused element and all its children up to MAX_RECURSION_DEPTH.
/// This handles cases where the focused element is a container (e.g., Chrome/Electron).
fn get_ax_selection_recursive() -> AppResult<Option<String>> {
    unsafe {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXUIElementCreateSystemWide() -> id;
            fn AXUIElementCopyAttributeValue(
                element: id,
                attribute: id,
                value: *mut id,
            ) -> i32;
            fn CFRelease(cf: id);
        }

        // Constants for Accessibility API
        let kAXFocusedApplicationAttribute = cocoa::foundation::NSString::alloc(nil)
            .init_str("AXFocusedApplication");
        let kAXFocusedUIElementAttribute = cocoa::foundation::NSString::alloc(nil)
            .init_str("AXFocusedUIElement");

        let system_element = AXUIElementCreateSystemWide();
        if system_element == nil {
            return Err(AppError::Io("Failed to create system-wide accessibility element".to_string()));
        }

        // Get focused application
        let mut focused_app: id = nil;
        let app_result = AXUIElementCopyAttributeValue(
            system_element,
            kAXFocusedApplicationAttribute,
            &mut focused_app,
        );

        if app_result != 0 || focused_app == nil {
            // No focused application or permission denied
            // system_element is created by AXUIElementCreateSystemWide, must be released
            if system_element != nil {
                CFRelease(system_element);
            }
            return Ok(None);
        }

        // Get focused UI element within the application
        let mut focused_element: id = nil;
        let element_result = AXUIElementCopyAttributeValue(
            focused_app,
            kAXFocusedUIElementAttribute,
            &mut focused_element,
        );

        if element_result != 0 || focused_element == nil {
            // No focused element - release what we have
            // focused_app and system_element are AXUIElementRef, must be released
            if focused_app != nil {
                CFRelease(focused_app);
            }
            if system_element != nil {
                CFRelease(system_element);
            }
            return Ok(None);
        }

        // Start recursive search from focused element
        let result = find_selection_in_element(focused_element, MAX_RECURSION_DEPTH);
        
        // Release AXUIElementRef objects we got from CopyAttributeValue
        // These are CoreFoundation types, not Objective-C objects
        if focused_element != nil {
            CFRelease(focused_element);
        }
        if focused_app != nil {
            CFRelease(focused_app);
        }
        if system_element != nil {
            CFRelease(system_element);
        }
        
        result
    }
}

/// Recursively search for selected text in an element and its children
/// 
/// # Arguments
/// * `element` - The AXUIElement to search
/// * `depth` - Remaining recursion depth (stops at 0)
/// 
/// # Returns
/// * `Some(String)` if selected text is found
/// * `None` if not found or depth limit reached
fn find_selection_in_element(element: id, depth: u8) -> AppResult<Option<String>> {
    if element == nil {
        return Ok(None);
    }

    // Safety: All AX API calls are wrapped in unsafe, but we validate pointers before use
    unsafe {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXUIElementCopyAttributeValue(
                element: id,
                attribute: id,
                value: *mut id,
            ) -> i32;
            fn CFRelease(cf: id);
        }

        // Constants
        let kAXSelectedTextAttribute = cocoa::foundation::NSString::alloc(nil)
            .init_str("AXSelectedText");
        let kAXChildrenAttribute = cocoa::foundation::NSString::alloc(nil)
            .init_str("AXChildren");

        // Step 1: Check if current element has selected text
        let mut selected_text: id = nil;
        let text_result = AXUIElementCopyAttributeValue(
            element,
            kAXSelectedTextAttribute,
            &mut selected_text,
        );

        // Handle selected_text result
        // Note: selected_text is an NSString (Objective-C object) returned by AX API
        // It's managed by autoreleasepool, so we MUST NOT call CFRelease on it
        if text_result == 0 && selected_text != nil {
            // Success - try to extract text
            let utf8_ptr = NSString::UTF8String(selected_text);
            if utf8_ptr != std::ptr::null() {
                // Safe conversion - UTF8String returns a pointer to internal buffer
                // We don't own this pointer, so we just read from it
                let text = match std::ffi::CStr::from_ptr(utf8_ptr).to_str() {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        // Invalid UTF-8, try lossy conversion
                        std::ffi::CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned()
                    }
                };

                // DO NOT release selected_text - it's an Objective-C object managed by autoreleasepool

                if !text.trim().is_empty() {
                    println!("[FindSelection] ✅ Found selected text at depth {} (length: {})", 
                        MAX_RECURSION_DEPTH - depth, text.len());
                    return Ok(Some(text));
                }
            }
        }

        // Step 2: If depth > 0, search children recursively
        if depth == 0 {
            return Ok(None);
        }

        // Get children
        let mut children: id = nil;
        let children_result = AXUIElementCopyAttributeValue(
            element,
            kAXChildrenAttribute,
            &mut children,
        );

        if children_result != 0 || children == nil {
            return Ok(None);
        }

        // Iterate through children using CFArray functions
        // Note: children is an NSArray (Objective-C object) returned by AX API
        // It's managed by autoreleasepool, so we MUST NOT call CFRelease on it
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFArrayGetCount(theArray: id) -> usize;
            fn CFArrayGetValueAtIndex(theArray: id, idx: usize) -> id;
        }
        
        let mut result: Option<String> = None;
        
        // Safely get count and iterate
        if children != nil {
            let count = CFArrayGetCount(children);
            
            // Limit iteration to prevent infinite loops
            let max_iterations = count.min(1000); // Safety limit
            
            for i in 0..max_iterations {
                let child = CFArrayGetValueAtIndex(children, i);
                if child != nil {
                    // Recursively search child
                    // Note: child is an AXUIElementRef (CoreFoundation type), not owned here
                    match find_selection_in_element(child, depth - 1) {
                        Ok(Some(text)) => {
                            result = Some(text);
                            break;
                        }
                        Err(e) => {
                            // Log error but continue searching other children
                            eprintln!("[FindSelection] Error searching child {}: {:?}", i, e);
                        }
                        _ => {}
                    }
                }
            }
            
            // DO NOT release children - it's an Objective-C NSArray managed by autoreleasepool
        }
        
        Ok(result)
    }
}

/// Fallback: Capture text via simulated Cmd+C with clipboard polling
/// 
/// Memory-safe implementation wrapped in autoreleasepool to prevent leaks/crashes
/// Uses correct NSString class reference to avoid nil pointer crashes
fn capture_via_simulated_copy(ignore_flag: Option<Arc<AtomicBool>>) -> AppResult<Option<String>> {
    // Wrap EVERYTHING in an autoreleasepool to prevent leaks/crashes
    autoreleasepool(|| {
        unsafe {
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGEventSourceCreate(stateID: u32) -> id;
                fn CGEventCreateKeyboardEvent(source: id, keyCode: u16, keyDown: bool) -> id;
                fn CGEventSetFlags(event: id, flags: u64);
                fn CGEventPost(tap: u32, event: id) -> ();
            }

            #[link(name = "CoreFoundation", kind = "framework")]
            extern "C" {
                fn CFRelease(cf: id);
            }

            // Step 1: Get initial count safely
            let pb: id = msg_send![class!(NSPasteboard), generalPasteboard];
            if pb == nil {
                return Err(AppError::Io("Failed to get NSPasteboard".to_string()));
            }

            let start_count: NSUInteger = msg_send![pb, changeCount];
            println!("[CaptureViaCopy] Initial clipboard change count: {}", start_count);

            // Step 2: Simulate Cmd+C (isolate in block)
            {
                // Set ignore flag before simulating keypress
                if let Some(flag) = &ignore_flag {
                    flag.store(true, Ordering::SeqCst);
                    println!("[CaptureViaCopy] 🚩 Ignore flag set for simulated Cmd+C");
                }

                // CGEventSourceStateID::HIDSystemState = 1
                let source = CGEventSourceCreate(1);
                if source == nil {
                    return Err(AppError::Io("Failed to create CGEventSource".to_string()));
                }

                // CMD key code = 0x37, 'C' key code = 0x08
                // CGEventFlagCommand = 0x100000
                // CGEventTapLocation::HIDEventTap = 0

                // CMD DOWN
                let cmd_down = CGEventCreateKeyboardEvent(source, 0x37, true);
                if cmd_down != nil {
                    CGEventSetFlags(cmd_down, 0x100000);
                    CGEventPost(0, cmd_down);
                    CFRelease(cmd_down); // SAFE: CGEventRef must be released
                }

                // 'C' DOWN
                let c_down = CGEventCreateKeyboardEvent(source, 0x08, true);
                if c_down != nil {
                    CGEventSetFlags(c_down, 0x100000);
                    CGEventPost(0, c_down);
                    CFRelease(c_down); // SAFE
                }

                // 'C' UP
                let c_up = CGEventCreateKeyboardEvent(source, 0x08, false);
                if c_up != nil {
                    CGEventSetFlags(c_up, 0x100000);
                    CGEventPost(0, c_up);
                    CFRelease(c_up); // SAFE
                }

                // CMD UP
                let cmd_up = CGEventCreateKeyboardEvent(source, 0x37, false);
                if cmd_up != nil {
                    CGEventPost(0, cmd_up);
                    CFRelease(cmd_up); // SAFE
                }

                CFRelease(source); // SAFE: CGEventSourceRef must be released
            }

            println!("[CaptureViaCopy] Polling for clipboard change...");

            // Step 3: Polling Loop
            for attempt in 1..=MAX_CLIPBOARD_POLL_ATTEMPTS {
                thread::sleep(Duration::from_millis(CLIPBOARD_POLL_INTERVAL_MS));

                // Re-acquire pasteboard inside pool
                let pb: id = msg_send![class!(NSPasteboard), generalPasteboard];
                if pb == nil {
                    continue;
                }

                let new_count: NSUInteger = msg_send![pb, changeCount];

                if new_count > start_count {
                    println!("[CaptureViaCopy] ✅ Clipboard changed detected!");

                    // CORRECT WAY to get NSString Class
                    let ns_string_class = class!(NSString);
                    
                    // Create array with the actual Class object
                    // Class is a pointer type, so we can use it directly
                    let classes: id = msg_send![class!(NSArray), arrayWithObject:ns_string_class as *const _ as id];
                    if classes == nil {
                        eprintln!("[CaptureViaCopy] ⚠️ Failed to create NSArray");
                        return Ok(None);
                    }

                    let strings: id = msg_send![pb, readObjectsForClasses:classes options:nil];
                    if strings == nil {
                        eprintln!("[CaptureViaCopy] ⚠️ Failed to read clipboard objects");
                        return Ok(None);
                    }

                    let count: NSUInteger = msg_send![strings, count];
                    if count > 0 {
                        let ns_str: id = msg_send![strings, objectAtIndex:0];

                        // Safety Checks
                        if ns_str == nil {
                            println!("[CaptureViaCopy] ⚠️ Initial object is nil");
                            return Ok(None);
                        }

                        let utf8_ptr = NSString::UTF8String(ns_str);
                        if utf8_ptr == std::ptr::null() {
                            println!("[CaptureViaCopy] ⚠️ UTF8String returned NULL");
                            return Ok(None);
                        }

                        let rust_str = CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned();
                        
                        if !rust_str.trim().is_empty() {
                            println!("[CaptureViaCopy] Captured text (length: {})", rust_str.len());
                            return Ok(Some(rust_str));
                        }
                    }
                    return Ok(None);
                }

                if attempt == MAX_CLIPBOARD_POLL_ATTEMPTS {
                    println!("[CaptureViaCopy] ❌ Timeout waiting for clipboard");
                }
            }

            Ok(None)
        }
    })
}

/// Smart selection detection (legacy - uses Accessibility API)
pub async fn detect_text_selection(_app: &tauri::AppHandle, ignore_flag: Option<Arc<AtomicBool>>) -> AppResult<(bool, Option<String>)> {
    match capture_selection(ignore_flag) {
        Ok(Some(text)) => Ok((true, Some(text))),
        Ok(None) => Ok((false, None)),
        Err(e) => Err(e),
    }
}
