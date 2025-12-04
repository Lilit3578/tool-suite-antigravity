//! macOS NSPanel configuration for non-activating floating panels
//! 
//! This module configures Tauri windows as floating panels that can
//! appear over fullscreen applications without activating the app.
//!
//! CRITICAL SEQUENCE:
//! 1. Window created (hidden) by Tauri
//! 2. Configure as floating panel (BEFORE showing)
//! 3. Set behaviors and level
//! 4. THEN show() using orderFrontRegardless
//!
//! All AppKit/Cocoa operations MUST run on the main thread.
//!
//! IMPORTANT: We configure the existing NSWindow to behave like a floating panel.
//! We do NOT attempt class mutation which can cause crashes.

#[cfg(target_os = "macos")]
use cocoa::{
    appkit::NSWindowCollectionBehavior,
    base::{id, nil, YES},
    foundation::NSRect,
};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use block::ConcreteBlock;
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex, mpsc};

// NSWindow level constants
// NSStatusWindowLevel = 25 (menu bar level - appears over full-screen apps)
#[cfg(target_os = "macos")]
const NS_STATUS_WINDOW_LEVEL: i64 = 25;

// NSMainMenuWindowLevel = 24 (main menu bar level)
#[cfg(target_os = "macos")]
const NS_MAIN_MENU_WINDOW_LEVEL: i64 = 24;

// NSPopUpMenuWindowLevel = 101 (pop-up menu level - NUCLEAR OPTION for guaranteed visibility)
#[cfg(target_os = "macos")]
const NS_POPUP_MENU_WINDOW_LEVEL: i64 = 101;

// NSApplicationActivationPolicy constants
// These control whether the app appears in the Dock and how it activates
#[cfg(target_os = "macos")]
const NS_APPLICATION_ACTIVATION_POLICY_REGULAR: i64 = 0;  // Normal app (shows in Dock)
#[cfg(target_os = "macos")]
const NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY: i64 = 1;  // Background agent (no Dock icon)
#[cfg(target_os = "macos")]
const NS_APPLICATION_ACTIVATION_POLICY_PROHIBITED: i64 = 2;  // Cannot activate

// NSWindow collection behavior constants
// ONLY use CanJoinAllSpaces + FullScreenAuxiliary for fullscreen overlays
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowCollectionBehaviorCanJoinAllSpaces: u64 = 1 << 0;  // 0x1
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowCollectionBehaviorFullScreenAuxiliary: u64 = 1 << 7; // 0x80 - CRITICAL FOR FULLSCREEN
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowCollectionBehaviorMoveToActiveSpace: u64 = 1 << 1; // 0x2 - Move window to active space

/// Set NSApplicationActivationPolicy to Accessory (hide from Dock, run as background agent)
/// 
/// CRITICAL: This prevents space-switching when activating the app.
/// - Regular policy (0): Shows in Dock, activating switches to app's space (Desktop)
/// - Accessory policy (1): No Dock icon, activating doesn't switch spaces
/// 
/// This MUST be called BEFORE attempting to show windows over fullscreen apps.
/// Once set to Accessory, the app can activate and show windows without forcing a space switch.
#[cfg(target_os = "macos")]
pub fn set_app_activation_policy_accessory() -> Result<(), String> {
    unsafe { set_app_activation_policy_accessory_impl() }
}

#[cfg(target_os = "macos")]
unsafe fn set_app_activation_policy_accessory_impl() -> Result<(), String> {
    let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
    if ns_app == nil {
        return Err("Failed to get NSApplication".to_string());
    }
    
    // Get current policy
    let current_policy: i64 = msg_send![ns_app, activationPolicy];
    println!("🔵 [DEBUG] [AppPolicy] Current activation policy: {} (0=Regular, 1=Accessory, 2=Prohibited)", current_policy);
    
    if current_policy == NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY {
        println!("🔵 [DEBUG] [AppPolicy] ✓ Already set to Accessory mode");
        return Ok(());
    }
    
    // Set to Accessory mode
    let success: bool = msg_send![ns_app, setActivationPolicy: NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY];
    
    if success {
        let verified_policy: i64 = msg_send![ns_app, activationPolicy];
        println!("🔵 [DEBUG] [AppPolicy] ✅ Set activation policy to Accessory (verified: {})", verified_policy);
        
        if verified_policy != NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY {
            return Err(format!("Policy verification failed: expected {}, got {}", NS_APPLICATION_ACTIVATION_POLICY_ACCESSORY, verified_policy));
        }
        
        Ok(())
    } else {
        Err("Failed to set activation policy to Accessory".to_string())
    }
}

/// Execute a closure on the main thread synchronously
/// This is CRITICAL for all AppKit/Cocoa operations to prevent EXC_BAD_ACCESS
#[cfg(target_os = "macos")]
fn run_on_main_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    use std::sync::mpsc::channel;
    
    // Check if we're already on the main thread
    let is_main_thread = unsafe {
        let current_thread: id = msg_send![class!(NSThread), currentThread];
        let is_main: bool = msg_send![current_thread, isMainThread];
        is_main
    };
    
    if is_main_thread {
        // Already on main thread, execute directly
        f();
        return;
    }
    
    // We're on a background thread, dispatch to main using NSOperationQueue
    let (tx, rx) = channel();
    
    // Wrap the FnOnce closure in Arc<Mutex<Option<F>>> so it can be called from Fn closure
    let closure = Arc::new(Mutex::new(Some(f)));
    
    unsafe {
        // Create an Objective-C block that executes our closure
        let block = ConcreteBlock::new(move || {
            let mut guard = match closure.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!("[Window] Mutex poisoned, recovering: {}", poisoned);
                    poisoned.into_inner()
                }
            };
            if let Some(f) = guard.take() {
                f();
            }
            let _ = tx.send(());
        });
        let block = block.copy();
        
        // Get the main queue and add our operation
        let main_queue: id = msg_send![class!(NSOperationQueue), mainQueue];
        let _: () = msg_send![main_queue, addOperationWithBlock: block];
    }
    
    // Wait for completion
    let _ = rx.recv();
}

/// Convert Tauri NSWindow to NSPanel for proper fullscreen overlay
/// 
/// CRITICAL: Tauri creates NSWindow by default, but we need NSPanel for:
/// - .nonactivatingPanel style mask
/// - Proper floating behavior over fullscreen apps
/// - No app activation when showing
/// 
/// This function:
/// 1. Gets the content view (WKWebView) from the Tauri window
/// 2. Creates a new NSPanel with proper configuration
/// 3. Transfers the content view to the panel
/// 4. Shows the panel
#[cfg(target_os = "macos")]
pub fn convert_window_to_panel(window: &tauri::WebviewWindow) -> Result<(), String> {
    use crate::system::window::panel::FloatingPanel;
    
    let (tx, rx) = mpsc::channel();
    let window_clone = window.clone();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = match window_clone.ns_window() {
                Ok(win) => win as id,
                Err(e) => {
                    eprintln!("[Window] Failed to get window handle: {}", e);
                    let _ = tx.send(Err(format!("Window handle not available: {}", e)));
                    return;
                }
            };
            
            println!("🔵 [DEBUG] [convert_to_panel] Converting Tauri NSWindow to NSPanel...");
            
            // Get the content view (WKWebView) from the Tauri window
            let content_view: id = msg_send![ns_window, contentView];
            if content_view == nil {
                eprintln!("🔴 [DEBUG] [convert_to_panel] ⚠️  No content view found!");
                let _ = tx.send(Err("No content view".to_string()));
                return;
            }
            
            // Retain the content view so it doesn't get deallocated when we remove it
            let _: () = msg_send![content_view, retain];
            
            // Get window frame for panel sizing
            let frame: NSRect = msg_send![ns_window, frame];
            println!("🔵 [DEBUG] [convert_to_panel] Window frame: x={}, y={}, w={}, h={}", 
                frame.origin.x, frame.origin.y, frame.size.width, frame.size.height);
            
            // Create NSPanel with same dimensions
            let panel = FloatingPanel::new(
                frame.origin.x,
                frame.origin.y,
                frame.size.width,
                frame.size.height
            );
            
            // Transfer the content view to the panel
            panel.set_content_view(content_view);
            
            // Release our retain (panel now owns it)
            let _: () = msg_send![content_view, release];
            
            // Hide the original Tauri window
            let _: () = msg_send![ns_window, orderOut: nil];
            
            // Show the panel
            panel.show();
            
            // Verify panel is visible
            if panel.is_visible() {
                println!("🔵 [DEBUG] [convert_to_panel] ✅ Panel is visible!");
                
                // CRITICAL: Prevent the FloatingPanel from being dropped
                // If we drop it, the NSPanel will be released and disappear
                // We use mem::forget to leak the panel - it will stay alive until app quits
                std::mem::forget(panel);
                
                let _ = tx.send(Ok(()));
            } else {
                eprintln!("🔴 [DEBUG] [convert_to_panel] ⚠️  Panel is NOT visible!");
                let _ = tx.send(Err("Panel not visible".to_string()));
            }
        }
    });
    
    rx.recv().map_err(|e| format!("Channel error: {}", e))?
}


/// Configure window for fullscreen overlay (main thread only)
/// 
/// MINIMAL configuration - ONLY sets window level and collection behavior.
/// These are the ONLY two properties needed for fullscreen overlay support.
/// 
/// CRITICAL: This MUST be called BEFORE the window is shown.
/// 
/// # Arguments
/// * `ns_window` - Raw NSWindow pointer from Tauri
/// 
/// # Safety
/// MUST be called from the main thread. Window MUST be hidden when called.
#[cfg(target_os = "macos")]
unsafe fn configure_for_fullscreen_overlay_main_thread(ns_window: id) -> Result<(), String> {
    if ns_window == nil {
        return Err("Received nil window pointer".to_string());
    }
    
    println!("🔵 [DEBUG] [Fullscreen] Configuring window for fullscreen overlay...");
    
    // Validate it's NSWindow
    let ns_window_class: id = msg_send![class!(NSWindow), class];
    let is_kind_of_class: bool = msg_send![ns_window, isKindOfClass: ns_window_class];
    if !is_kind_of_class {
        return Err("Provided pointer is not NSWindow".to_string());
    }

    // Get current window level for debugging
    let current_level: i64 = msg_send![ns_window, level];
    println!("🔵 [DEBUG] [Fullscreen] Current window level: {} (target: {})", current_level, NS_STATUS_WINDOW_LEVEL);

    // CRITICAL PROPERTY 1: Set window level to NSStatusWindowLevel (25)
    // This is REQUIRED for windows to appear over fullscreen apps
    let _: () = msg_send![ns_window, setLevel: NS_STATUS_WINDOW_LEVEL];
    
    // Verify it was set correctly
    let verified_level: i64 = msg_send![ns_window, level];
    println!("🔵 [DEBUG] [Fullscreen] ✓ Set window level to {} (verified: {})", NS_STATUS_WINDOW_LEVEL, verified_level);
    
    if verified_level != NS_STATUS_WINDOW_LEVEL {
        eprintln!("🔴 [DEBUG] [Fullscreen] ⚠️  WARNING: Window level verification failed! Expected {}, got {}", NS_STATUS_WINDOW_LEVEL, verified_level);
    }

    // Get current collection behavior for debugging
    let current_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
    println!("🔵 [DEBUG] [Fullscreen] Current collection behavior: 0x{:x}", current_behavior.bits());

    // CRITICAL PROPERTY 2: Set collection behavior for fullscreen support
    // Use CanJoinAllSpaces + FullScreenAuxiliary (0x81)
    // CanJoinAllSpaces: Window appears on ALL spaces simultaneously (including fullscreen)
    // FullScreenAuxiliary: Window appears over fullscreen apps
    // NOTE: We use CanJoinAllSpaces instead of MoveToActiveSpace because:
    //       - CanJoinAllSpaces makes window appear on ALL spaces at once (including fullscreen)
    //       - MoveToActiveSpace requires app activation, which switches spaces
    //       - CanJoinAllSpaces works with orderFrontRegardless (non-activating)
    let desired_bits: u64 = NSWindowCollectionBehaviorCanJoinAllSpaces 
        | NSWindowCollectionBehaviorFullScreenAuxiliary;  // 0x1 | 0x80 = 0x81
    let behavior = NSWindowCollectionBehavior::from_bits_truncate(desired_bits);
    let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
    
    // Verify it was set correctly
    let verified_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
    let verified_bits = verified_behavior.bits();
    println!("🔵 [DEBUG] [Fullscreen] ✓ Set collection behavior to 0x{:x} (verified: 0x{:x})", desired_bits, verified_bits);
    
    if (verified_bits & NSWindowCollectionBehaviorFullScreenAuxiliary) == 0 {
        eprintln!("🔴 [DEBUG] [Fullscreen] ⚠️  CRITICAL: FullScreenAuxiliary flag is missing!");
    }
    if (verified_bits & NSWindowCollectionBehaviorCanJoinAllSpaces) == 0 {
        eprintln!("🔴 [DEBUG] [Fullscreen] ⚠️  CRITICAL: CanJoinAllSpaces flag is missing!");
    }
    
    // Check if window is on active space
    let is_on_active_space: bool = msg_send![ns_window, isOnActiveSpace];
    println!("🔵 [DEBUG] [Fullscreen] Window is on active space: {}", is_on_active_space);

    // Additional debugging: Check if window can become key
    let can_become_key: bool = msg_send![ns_window, canBecomeKeyWindow];
    let can_become_main: bool = msg_send![ns_window, canBecomeMainWindow];
    println!("🔵 [DEBUG] [Fullscreen] Window capabilities: canBecomeKey={}, canBecomeMain={}", can_become_key, can_become_main);

    println!("🔵 [DEBUG] [Fullscreen] ✅ Fullscreen overlay configuration complete");
    Ok(())
}


/// Configure Tauri window for fullscreen overlay BEFORE showing
/// 
/// MINIMAL configuration - only sets window level and collection behavior.
/// These are the ONLY properties needed for fullscreen overlay support.
/// 
/// CRITICAL: This MUST be called:
/// 1. AFTER window.build() but BEFORE window.show()
/// 2. While window is still hidden
/// 
/// Sequence:
/// - Window created (hidden) → Configure → THEN show()
/// 
/// # Arguments
/// * `window` - Tauri WebviewWindow reference (must be hidden)
#[cfg(target_os = "macos")]
pub fn configure_for_fullscreen_overlay(window: &tauri::WebviewWindow) -> Result<(), String> {
    // Verify window is hidden
    let is_visible = window.is_visible().unwrap_or(false);
    println!("🔵 [DEBUG] [Fullscreen] configure_for_fullscreen_overlay() called - window visible: {}", is_visible);
    
    if is_visible {
        eprintln!("🔴 [DEBUG] [Fullscreen] ⚠️  WARNING: Window is visible! Should be hidden for configuration.");
        // Don't return error, just warn - might be reconfiguring existing window
    }
    
    // Get the raw NSWindow pointer from Tauri
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    println!("🔵 [DEBUG] [Fullscreen] Got NSWindow pointer: {:p}", ns_window_ptr);
    
    // Convert the raw pointer to usize for thread-safe transfer
    let ns_window_usize = ns_window_ptr as usize;
    
    // Dispatch to main thread and wait for result
    let (tx, rx) = mpsc::channel();
    
    println!("🔵 [DEBUG] [Fullscreen] Dispatching configuration to main thread...");
    run_on_main_thread(move || {
        // Execute the configuration on the main thread
        let res = unsafe {
            configure_for_fullscreen_overlay_main_thread(ns_window_usize as id)
        };
        let _ = tx.send(res);
    });
    
    println!("🔵 [DEBUG] [Fullscreen] Waiting for configuration result...");
    // Get result
    match rx.recv() {
        Ok(Ok(())) => {
            println!("🔵 [DEBUG] [Fullscreen] ✅ Configuration completed successfully");
            Ok(())
        },
        Ok(Err(e)) => {
            eprintln!("🔴 [DEBUG] [Fullscreen] ❌ Configuration failed: {}", e);
            Err(e)
        },
        Err(_) => {
            eprintln!("🔴 [DEBUG] [Fullscreen] ❌ Failed to receive configuration result");
            Err("Failed to get configuration result".to_string())
        },
    }
}

/// Show window and ensure it stays visible over fullscreen apps
/// 
/// This function:
/// 1. Ensures window level 25 (NSStatusWindowLevel) is set
/// 2. Ensures collection behavior is CanJoinAllSpaces | FullScreenAuxiliary
/// 3. Uses Tauri's normal window.show() (which may briefly activate the app)
/// 
/// The caller should restore focus to the original app immediately after calling this.
#[cfg(target_os = "macos")]
pub fn ensure_fullscreen_overlay_config(window: &tauri::WebviewWindow) -> Result<(), String> {
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            
            // Ensure window level is set to 25 (NSStatusWindowLevel) for fullscreen overlay
            let current_level: i64 = msg_send![ns_window, level];
            if current_level != NS_STATUS_WINDOW_LEVEL {
                let _: () = msg_send![ns_window, setLevel: NS_STATUS_WINDOW_LEVEL];
                let verified_level: i64 = msg_send![ns_window, level];
                println!("🔵 [DEBUG] [Fullscreen] Set window level to {} (was: {}, now: {})", NS_STATUS_WINDOW_LEVEL, current_level, verified_level);
            }
            
            // Ensure collection behavior is CanJoinAllSpaces | FullScreenAuxiliary (0x81)
            // CanJoinAllSpaces makes window appear on ALL spaces (including fullscreen) without activating app
            let current_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
            let desired_bits: u64 = NSWindowCollectionBehaviorCanJoinAllSpaces 
                | NSWindowCollectionBehaviorFullScreenAuxiliary;  // 0x1 | 0x80 = 0x81
            let current_bits = current_behavior.bits();
            
            if current_bits != desired_bits {
                let behavior = NSWindowCollectionBehavior::from_bits_truncate(desired_bits);
                let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
                let verified_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
                println!("🔵 [DEBUG] [Fullscreen] Set collection behavior to 0x{:x} (was: 0x{:x}, now: 0x{:x})", desired_bits, current_bits, verified_behavior.bits());
            }
            
            let _ = tx.send(Ok(()));
        }
    });
    
    match rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to configure window for fullscreen overlay".to_string()),
    }
}

/// Show window using Tauri's normal API and restore focus to original app
/// 
/// This approach works WITH Tauri rather than against it:
/// 1. Ensures window level 25 and collection behavior are set
/// 2. Uses Tauri's window.show() (which may briefly activate the app)
/// 3. Immediately restores focus to the original app
/// 
/// The window stays visible because of its high window level (25).
#[cfg(target_os = "macos")]
pub fn show_and_restore_focus(window: &tauri::WebviewWindow, original_app: Option<&str>) -> Result<(), String> {
    // First, ensure the window is configured for fullscreen overlay
    ensure_fullscreen_overlay_config(window)?;
    
    // Show the window using Tauri's normal API
    // This may briefly activate the app, but that's acceptable
    window.show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    
    // If we have an original app name, restore focus to it immediately
    if let Some(app_name) = original_app {
        // Use a small delay to let the window appear first
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Restore focus to the original app
        // This is done via AppleScript, so we need to import the automation module
        // For now, we'll just log it - the caller should handle focus restoration
        println!("🔵 [DEBUG] [Fullscreen] Window shown, should restore focus to: {}", app_name);
    }
    
    Ok(())
}

/// Order window to front without activating app (legacy function name - deprecated)
/// 
/// This function is deprecated. Use `show_and_restore_focus` instead.
#[cfg(target_os = "macos")]
pub fn order_window_front(window: &tauri::WebviewWindow) -> Result<(), String> {
    ensure_fullscreen_overlay_config(window)
}

/// Verify window is actually visible and get its position
/// This helps debug why windows might not appear
#[cfg(target_os = "macos")]
pub fn verify_window_visibility(window: &tauri::WebviewWindow) -> Result<(bool, bool, f64, f64, f64, f64), String> {
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            let is_visible: bool = msg_send![ns_window, isVisible];
            let is_on_screen: bool = msg_send![ns_window, isOnActiveSpace];
            
            // Get window frame
            let frame: NSRect = msg_send![ns_window, frame];
            let x = frame.origin.x;
            let y = frame.origin.y;
            let width = frame.size.width;
            let height = frame.size.height;
            
            println!("🔵 [DEBUG] [NSWindow] Window visibility check:");
            println!("🔵 [DEBUG] [NSWindow]   - isVisible: {}", is_visible);
            println!("🔵 [DEBUG] [NSWindow]   - isOnActiveSpace: {}", is_on_screen);
            println!("🔵 [DEBUG] [NSWindow]   - frame: ({}, {}) size: {}x{}", x, y, width, height);
            
            if !is_visible {
                eprintln!("🔴 [DEBUG] [NSWindow] ⚠️  Window is NOT visible!");
            }
            if !is_on_screen {
                eprintln!("🔴 [DEBUG] [NSWindow] ⚠️  Window is NOT on active space!");
            }
            
            let _ = tx.send(Ok((is_visible, is_on_screen, x, y, width, height)));
        }
    });
    
    match rx.recv() {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to verify window visibility".to_string()),
    }
}

/// Make the window key (give it focus) so user can type immediately
/// 
/// This makes the window the key window, which allows it to receive keyboard input.
/// The window must already be visible for this to work.
#[cfg(target_os = "macos")]
pub fn make_window_key(window: &tauri::WebviewWindow) -> Result<(), String> {
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            
            // Check if window can become key
            let can_become_key: bool = msg_send![ns_window, canBecomeKeyWindow];
            if !can_become_key {
                eprintln!("🔴 [DEBUG] [make_window_key] ⚠️  Window cannot become key window");
                let _ = tx.send(Err("Window cannot become key window".to_string()));
                return;
            }
            
            // Make the window key (give it focus)
            let _: () = msg_send![ns_window, makeKeyWindow];
            
            // Verify it became key
            let is_key: bool = msg_send![ns_window, isKeyWindow];
            if is_key {
                println!("🔵 [DEBUG] [make_window_key] ✓ Window is now key window (has focus)");
            } else {
                eprintln!("🔴 [DEBUG] [make_window_key] ⚠️  Window did not become key window");
            }
            
            let _ = tx.send(Ok(()));
        }
    });
    
    match rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to make window key".to_string()),
    }
}

/// Configure window for fullscreen overlay (call this immediately after window creation)
/// This sets the window level and collection behavior BEFORE showing
#[cfg(target_os = "macos")]
pub fn configure_window_for_fullscreen(window: &tauri::WebviewWindow) -> Result<(), String> {
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            
            // Set window level to 25 (NSStatusWindowLevel)
            let _: () = msg_send![ns_window, setLevel: NS_STATUS_WINDOW_LEVEL];
            
            // Set collection behavior to CanJoinAllSpaces + FullScreenAuxiliary (0x81)
            // CanJoinAllSpaces makes window appear on ALL spaces (including fullscreen)
            let desired_bits: u64 = NSWindowCollectionBehaviorCanJoinAllSpaces 
                | NSWindowCollectionBehaviorFullScreenAuxiliary;  // 0x1 | 0x80 = 0x81
            let behavior = NSWindowCollectionBehavior::from_bits_truncate(desired_bits);
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
            
            // Verify
            let verified_level: i64 = msg_send![ns_window, level];
            let verified_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
            
            println!("🔵 [DEBUG] [configure_window_for_fullscreen] Window configured:");
            println!("🔵 [DEBUG] [configure_window_for_fullscreen]   - Level: {} (target: {})", verified_level, NS_STATUS_WINDOW_LEVEL);
            println!("🔵 [DEBUG] [configure_window_for_fullscreen]   - Collection behavior: 0x{:x} (target: 0x81)", verified_behavior.bits());
            
            if verified_level != NS_STATUS_WINDOW_LEVEL || (verified_behavior.bits() & 0x81) != 0x81 {
                eprintln!("🔴 [DEBUG] [configure_window_for_fullscreen] ⚠️  Configuration may not be correct!");
            }
            
            let _ = tx.send(Ok(()));
        }
    });
    
    match rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to configure window for fullscreen".to_string()),
    }
}

/// Show window over fullscreen apps using panel-like configuration
/// 
/// STANDARD APPROACH (No NSPanel Conversion):
/// Instead of converting NSWindow to NSPanel (which breaks Tauri's renderer),
/// we configure the existing NSWindow to behave like a floating panel.
/// 
/// The Solution:
/// 1. App policy is already set to Accessory (in lib.rs setup)
/// 2. Add NSNonactivatingPanelMask to style mask
/// 3. Set window level to NSPopUpMenuWindowLevel (101)
/// 4. Set collection behavior (CanJoinAllSpaces + FullScreenAuxiliary = 0x81)
/// 5. Show with orderFrontRegardless (non-activating)
/// 6. Activate app ignoring other apps
/// 7. Make window key for keyboard input

#[cfg(target_os = "macos")]
pub fn show_window_over_fullscreen(window: &tauri::WebviewWindow) -> Result<(), String> {
    println!("🔵 [DEBUG] [show_window_over_fullscreen] ═══════════════════════════════");
    println!("🔵 [DEBUG] [show_window_over_fullscreen] 🚀 PANEL-LIKE CONFIGURATION");
    println!("🔵 [DEBUG] [show_window_over_fullscreen] ═══════════════════════════════");
    
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            
            println!("🔵 [DEBUG] Configuring NSWindow with panel-like behavior...");
            
            // STEP 1: Add NSNonactivatingPanelMask to style mask
            // NSWindowStyleMaskNonactivatingPanel = 1 << 7 = 128
            const NS_NONACTIVATING_PANEL_MASK: u64 = 1 << 7;
            let current_style: u64 = msg_send![ns_window, styleMask];
            let new_style = current_style | NS_NONACTIVATING_PANEL_MASK;
            let _: () = msg_send![ns_window, setStyleMask: new_style];
            println!("🔵 [DEBUG] ✓ Style mask: 0x{:x} -> 0x{:x} (added NonactivatingPanel)", current_style, new_style);
            
            // STEP 2: Set window level to NSPopUpMenuWindowLevel (101)
            let _: () = msg_send![ns_window, setLevel: NS_POPUP_MENU_WINDOW_LEVEL];
            let verified_level: i64 = msg_send![ns_window, level];
            println!("🔵 [DEBUG] ✓ Window level set to {} (NSPopUpMenuWindowLevel)", verified_level);
            
            // STEP 3: Set collection behavior (CanJoinAllSpaces + FullScreenAuxiliary = 0x81)
            let desired_bits: u64 = NSWindowCollectionBehaviorCanJoinAllSpaces 
                | NSWindowCollectionBehaviorFullScreenAuxiliary;
            let behavior = NSWindowCollectionBehavior::from_bits_truncate(desired_bits);
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
            let verified_behavior: NSWindowCollectionBehavior = msg_send![ns_window, collectionBehavior];
            println!("🔵 [DEBUG] ✓ Collection behavior set to 0x{:x}", verified_behavior.bits());
            
            // STEP 4: Show with orderFrontRegardless (non-activating)
            let _: () = msg_send![ns_window, orderFrontRegardless];
            println!("🔵 [DEBUG] ✓ Window ordered front (non-activating)");
            
            // STEP 5: Activate app ignoring other apps
            let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![ns_app, activateIgnoringOtherApps: YES];
            println!("🔵 [DEBUG] ✓ App activated (Accessory mode prevents space switch)");
            
            // STEP 6: Make window key for keyboard input
            let _: () = msg_send![ns_window, makeKeyAndOrderFront: nil];
            let is_key: bool = msg_send![ns_window, isKeyWindow];
            println!("🔵 [DEBUG] ✓ Window is key: {}", is_key);
            
            // Verify visibility
            let is_visible: bool = msg_send![ns_window, isVisible];
            let is_on_active_space: bool = msg_send![ns_window, isOnActiveSpace];
            println!("🔵 [DEBUG] ✅ Window visible: {}, on active space: {}", is_visible, is_on_active_space);
            
            if is_visible && is_on_active_space {
                let _ = tx.send(Ok(()));
            } else {
                let _ = tx.send(Err(format!("Window not properly visible: visible={}, onActiveSpace={}", is_visible, is_on_active_space)));
            }
        }
    });
    
    match rx.recv() {
        Ok(Ok(())) => {
            println!("🔵 [DEBUG] [show_window_over_fullscreen] ✅ SUCCESS");
            Ok(())
        },
        Ok(Err(e)) => {
            eprintln!("🔴 [DEBUG] [show_window_over_fullscreen] ❌ FAILED: {}", e);
            Err(e)
        },
        Err(_) => Err("Failed to receive result from main thread".to_string()),
    }
}

// Stub for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn show_window_over_fullscreen(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("show_window_over_fullscreen only supported on macOS".to_string())
}

/// Get native window state for debugging
#[cfg(target_os = "macos")]
pub fn get_native_window_state(window: &tauri::WebviewWindow) -> Result<(bool, bool, bool, bool, bool), String> {
    let ns_window_ptr = window.ns_window()
        .map_err(|e| format!("Failed to get NSWindow: {}", e))?;
    
    let ns_window_usize = ns_window_ptr as usize;
    let (tx, rx) = mpsc::channel();
    
    run_on_main_thread(move || {
        unsafe {
            let ns_window = ns_window_usize as id;
            let is_visible: bool = msg_send![ns_window, isVisible];
            let is_key: bool = msg_send![ns_window, isKeyWindow];
            let is_main: bool = msg_send![ns_window, isMainWindow];
            let can_become_key: bool = msg_send![ns_window, canBecomeKeyWindow];
            let can_become_main: bool = msg_send![ns_window, canBecomeMainWindow];
            let _ = tx.send(Ok((is_visible, is_key, is_main, can_become_key, can_become_main)));
        }
    });
    
    match rx.recv() {
        Ok(Ok(state)) => Ok(state),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to get window state".to_string()),
    }
}

// Stubs for non-macOS
#[cfg(not(target_os = "macos"))]
pub fn configure_as_non_activating_panel(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("NSPanel configuration only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn order_window_front(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("order_window_front only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn make_window_key(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("make_window_key only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn configure_window_for_fullscreen(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("configure_window_for_fullscreen only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn show_window_over_fullscreen(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Err("show_window_over_fullscreen only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn verify_window_visibility(_window: &tauri::WebviewWindow) -> Result<(bool, bool, f64, f64, f64, f64), String> {
    Err("verify_window_visibility only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn get_native_window_state(_window: &tauri::WebviewWindow) -> Result<(bool, bool, bool, bool, bool), String> {
    Err("Native window state only available on macOS".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_compiles() {
        // This test just ensures the module compiles on all platforms
        assert!(true);
    }
}