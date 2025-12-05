// Module declarations - these are re-exported from their respective modules
mod shared;
mod api;
mod core;
mod system;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // CRITICAL: Set app activation policy to Accessory FIRST
            // This prevents space-switching when activating the app
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = system::window::nswindow::set_app_activation_policy_accessory() {
                    eprintln!("⚠️  Failed to set Accessory mode: {}", e);
                } else {
                    println!("✅ App set to Accessory mode (no Dock icon, no space switching)");
                }
            }
            
            // Load settings
            let _settings = shared::settings::AppSettings::load()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to load settings: {}", e);
                    shared::settings::AppSettings::default()
                });

            // Initialize clipboard history and monitor
            let clipboard_history = core::clipboard::history::ClipboardHistory::new();
            let clipboard_monitor = core::clipboard::monitor::ClipboardMonitor::new(clipboard_history.clone_arc());
            
            // Initialize usage metrics for intelligent ranking
            let usage_metrics = core::context::UsageMetrics::new();
            
            // Initialize last active app tracker
            let last_active_app = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
            
            // Initialize window operation lock to prevent race conditions
            let window_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
            
            // FIXED: Add debounce flag to prevent concurrent shortcut triggers
            let shortcut_debounce = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            
            // Store in app state for access from commands
            // CRITICAL: Must manage ALL state BEFORE accessing it in closures
            app.manage(clipboard_history);
            app.manage(clipboard_monitor.clone_arc());
            app.manage(usage_metrics);
            app.manage(last_active_app);
            app.manage(window_lock);
            app.manage(shortcut_debounce.clone());
            
            // Start clipboard monitoring
            clipboard_monitor.start(app.handle().clone());
            println!("✅ Clipboard monitoring started");

            // Create tray menu (Command Palette as single access point)
            let palette_item = MenuItem::with_id(app, "palette", "Open Command Palette", true, None::<&str>)?;
            let clipboard_item = MenuItem::with_id(app, "clipboard", "Clipboard History (5)", true, None::<&str>)?;
            let toggle_monitor_item = MenuItem::with_id(app, "toggle_monitor", "⏸ Pause Monitoring", true, None::<&str>)?;
            let clear_history_item = MenuItem::with_id(app, "clear_history", "Clear History", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &palette_item,
                    &separator,
                    &clipboard_item,
                    &toggle_monitor_item,
                    &clear_history_item,
                    &separator,
                    &settings_item,
                    &separator,
                    &quit_item,
                ],
            )?;

            // Build tray icon
            let default_icon = app.default_window_icon()
                .ok_or("Failed to get default window icon")?;
            let _tray = TrayIconBuilder::new()
                .icon(default_icon.clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "palette" => {
                            // FIXED: Spawn async task instead of calling sync function
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Some(window_lock) = app_handle.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
                                    if let Err(e) = show_widget_window_async(&app_handle, "palette", false, window_lock.inner().clone()).await {
                                eprintln!("Failed to show palette: {}", e);
                            }
                                } else {
                                    eprintln!("Window lock not available");
                                }
                            });
                        }
                        "clipboard" => {
                            // FIXED: Spawn async task instead of calling sync function
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Some(window_lock) = app_handle.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
                                    if let Err(e) = show_widget_window_async(&app_handle, "palette", false, window_lock.inner().clone()).await {
                                eprintln!("Failed to show clipboard history: {}", e);
                            }
                                } else {
                                    eprintln!("Window lock not available");
                                }
                            });
                        }
                        "toggle_monitor" => {
                            if let Some(monitor) = app.try_state::<core::clipboard::monitor::ClipboardMonitor>() {
                                let enabled = monitor.toggle();
                                println!("Clipboard monitoring: {}", if enabled { "enabled" } else { "disabled" });
                                
                                // Update menu item text
                                // Note: Tauri doesn't support dynamic menu text updates easily
                                // This would require rebuilding the tray menu
                            }
                        }
                        "clear_history" => {
                            if let Some(history) = app.try_state::<core::clipboard::history::ClipboardHistory>() {
                                history.clear();
                                println!("Clipboard history cleared");
                            }
                        }
                        "settings" => {
                            // FIXED: Spawn async task instead of calling sync function
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Some(window_lock) = app_handle.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
                                    if let Err(e) = show_widget_window_async(&app_handle, "settings", false, window_lock.inner().clone()).await {
                                eprintln!("Failed to show settings: {}", e);
                            }
                                } else {
                                    eprintln!("Window lock not available");
                                }
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        println!("Tray icon clicked!");
                    }
                })
                .build(app)?;

            // Register global shortcuts with retry logic
            let app_handle = app.handle().clone();
            
            // Parse and register command palette shortcut
            // Changed from Control+Shift+L to avoid conflicts with other apps
            let shortcut_str = "Control+Shift+Space";
            match shortcut_str.parse::<Shortcut>() {
                Ok(shortcut) => {
                    // First, unregister if already registered (clean slate)
                    if let Err(e) = app.global_shortcut().unregister(shortcut) {
                        // It's okay if unregister fails (shortcut might not be registered yet)
                        println!("ℹ️  Unregister attempt (expected on first run): {}", e);
                    } else {
                        println!("🔄 Unregistered existing shortcut: {}", shortcut_str);
                    }
                    
                    let handle = app_handle.clone();
                    let last_app_state = app.state::<std::sync::Arc<std::sync::Mutex<Option<String>>>>();
                    let last_app_clone = last_app_state.inner().clone();
                    // FIXED: Use the cloned debounce flag directly instead of accessing state
                    let debounce_clone = shortcut_debounce.clone();
                    
                    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                        // FIXED: Debounce to prevent concurrent triggers
                        if debounce_clone.swap(true, std::sync::atomic::Ordering::Acquire) {
                            println!("🔵 [DEBUG] Shortcut debounced - ignoring concurrent trigger");
                            return;
                        }
                        
                        println!("🔵 [DEBUG] ========== SHORTCUT TRIGGERED ==========");
                        
                        // Spawn async task to avoid blocking the shortcut handler
                        let handle_clone = handle.clone();
                        let last_app_clone = last_app_clone.clone();
                        let debounce_reset = debounce_clone.clone();
                        
                        // Reset debounce flag after 500ms
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            debounce_reset.store(false, std::sync::atomic::Ordering::Release);
                        });
                        
                        // Main shortcut handler task
                        tauri::async_runtime::spawn(async move {
                            println!("🔵 [DEBUG] [Shortcut] Async task started");
                            
                            // STEP 1: Capture active app BEFORE any operations
                            println!("🔵 [DEBUG] [Shortcut] STEP 1: Capturing active app...");
                            let initial_active_app = system::automation::macos::get_active_app().ok();
                            println!("🔵 [DEBUG] [Shortcut] Initial active app: {:?}", initial_active_app);
                            
                            // Store the active app for later paste operations
                            if let Some(ref active_app) = initial_active_app {
                                    match last_app_clone.lock() {
                                        Ok(mut last_app) => {
                                            *last_app = Some(active_app.clone());
                                        println!("🔵 [DEBUG] [Shortcut] Stored active app: {}", active_app);
                                        }
                                        Err(poisoned) => {
                                        eprintln!("🔴 [DEBUG] [Shortcut] Mutex poisoned, recovering...");
                                            let mut guard = poisoned.into_inner();
                                            *guard = Some(active_app.clone());
                                    }
                                }
                            }
                            
                            // STEP 2: Detect text selection BEFORE opening window
                            // This is the KEY FIX - we capture selection while original app still has focus
                            // FIXED: Store original app before detection to restore focus later
                            let original_app_before_detection = system::automation::macos::get_active_app().ok();
                            println!("🔵 [DEBUG] [Shortcut] STEP 2: Detecting text selection (BEFORE window creation)...");
                            println!("🔵 [DEBUG] [Shortcut] Original app before detection: {:?}", original_app_before_detection);
                            
                            let (has_selection, _selected_text) = match system::automation::macos::detect_text_selection(&handle_clone).await {
                                Ok(result) => {
                                    println!("🔵 [DEBUG] [Shortcut] ✓ Selection detection completed: has_selection={}", result.0);
                                    result
                                }
                                Err(e) => {
                                    eprintln!("🔴 [DEBUG] [Shortcut] ✗ Selection detection failed: {}", e);
                                    (false, None)
                                }
                            };
                            
                            // STEP 3: NOW open the palette window with the selection result
                            // Since we already captured the selection, there's no focus conflict
                            println!("🔵 [DEBUG] [Shortcut] STEP 3: Opening palette window (has_selection={})...", has_selection);
                            
                            // Get window lock from app state
                            if let Some(window_lock) = handle_clone.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
                                match show_widget_window_async(&handle_clone, "palette", has_selection, window_lock.inner().clone()).await {
                                    Ok(_) => {
                                        println!("🔵 [DEBUG] [Shortcut] ✓ Window opened successfully");
                                        
                                        // Just focus - no verification or retry (show_window_over_fullscreen handles retries internally)
                                        if let Some(window) = handle_clone.get_webview_window("palette-window") {
                                            // Increased delay for full transition to complete
                                            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
                                            
                                            // Set focus (safe during transition)
                                            if let Err(e) = window.set_focus() {
                                                eprintln!("🔴 [DEBUG] [Shortcut] ⚠️  Failed to give window focus: {}", e);
                                            } else {
                                                println!("🔵 [DEBUG] [Shortcut] ✓ Window has focus - ready for typing");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("🔴 [DEBUG] [Shortcut] ✗ Failed to show palette window: {}", e);
                                        return;
                                    }
                                }
                            } else {
                                eprintln!("🔴 [DEBUG] [Shortcut] ✗ Window lock not found in app state, using fallback");
                                // Fallback to sync version
                                if let Err(e) = show_widget_window(&handle_clone, "palette", has_selection) {
                                    eprintln!("🔴 [DEBUG] [Shortcut] ✗ Fallback also failed: {}", e);
                                } else {
                                    // Give window focus after showing
                                    if let Some(window) = handle_clone.get_webview_window("palette-window") {
                                        std::thread::sleep(std::time::Duration::from_millis(150));
                                        window.set_focus().ok();
                                    }
                                }
                            }
                            
                            // DONE: Window is shown and has focus, ready for user input
                            println!("🔵 [DEBUG] [Shortcut] ✅ Window shown with focus - ready for typing!");
                            
                            println!("🔵 [DEBUG] ========== SHORTCUT HANDLER COMPLETE ==========");
                        });
                    }) {
                        eprintln!("Failed to set handler for command palette shortcut: {}", e);
                    } else {
                        // Retry registration with exponential backoff
                        let mut registered = false;
                        let max_retries = 5;
                        
                        for attempt in 0..max_retries {
                            match app.global_shortcut().register(shortcut) {
                                Ok(_) => {
                                    println!("✅ Registered global shortcut: {} (attempt {})", shortcut_str, attempt + 1);
                                    registered = true;
                                    break;
                                }
                                Err(e) => {
                                    if attempt < max_retries - 1 {
                                        let delay_ms = 100 * (2_u64.pow(attempt as u32)); // Exponential backoff: 100ms, 200ms, 400ms, 800ms
                                        eprintln!("⚠️  Shortcut registration attempt {} failed: {}. Retrying in {}ms...", 
                                            attempt + 1, e, delay_ms);
                                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                                    } else {
                                        eprintln!("❌ Failed to register command palette shortcut after {} attempts: {}", max_retries, e);
                                        eprintln!("💡 Tip: Check System Settings > Keyboard > Keyboard Shortcuts for conflicts");
                                    }
                                }
                            }
                        }
                        
                        if !registered {
                            eprintln!("⚠️  App will continue without global shortcut. Use tray menu to access widgets.");
                        }
                    }
                }
                Err(e) => eprintln!("Failed to parse command palette shortcut '{}': {}", shortcut_str, e),
            }


            println!("✅ Productivity Widgets initialized successfully!");
            println!("📋 Global Shortcut: Control+Shift+Space");
            println!("💡 All widgets and actions accessible via Command Palette");
            
            // Listen for floating window event
            // In Tauri v2, we use a command handler instead of event listener
            // The frontend will call a command to open the floating window
            println!("✅ Floating panel ready (use command or emit event from frontend)");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Palette commands
            api::commands::palette::capture_selection,
            api::commands::palette::get_command_items,
            api::commands::palette::execute_action,
            api::commands::palette::record_command_usage,
            // Window commands
            api::commands::window::get_cursor_position,
            api::commands::window::get_primary_monitor_bounds,
            api::commands::window::calculate_palette_position,
            api::commands::window::hide_palette_window,
            api::commands::window::show_widget,
            // System commands
            api::commands::system::get_active_app,
            api::commands::system::check_accessibility_permissions,
            api::commands::system::log_message,
            // Settings commands
            api::commands::settings::get_settings,
            api::commands::settings::save_settings,
            // Feature commands
            core::features::translator::translate_text,
            core::features::currency::convert_currency,
            core::features::clipboard::get_clipboard_history,
            core::features::clipboard::paste_clipboard_item,
            core::features::clipboard::clear_clipboard_history,
            core::features::clipboard::toggle_clipboard_monitor,
            core::features::clipboard::get_clipboard_monitor_status,
            core::features::time_converter::convert_time,
            core::features::time_converter::get_timezones,
            core::features::time_converter::parse_time_from_selection,
            core::features::time_converter::get_system_timezone,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to start Tauri application: {}", e);
            eprintln!("This is a critical error. Please check logs and system permissions.");
            eprintln!("Common causes:");
            eprintln!("  - Missing system permissions (accessibility, screen recording)");
            eprintln!("  - Port conflicts or network issues");
            eprintln!("  - Corrupted application state");
            std::process::exit(1);
        });
}

// Async version with mutex lock to prevent race conditions
async fn show_widget_window_async(
    app: &tauri::AppHandle,
    widget: &str,
    has_selection: bool,
    window_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
) -> Result<(), String> {
    // CRITICAL: Acquire lock to prevent concurrent operations
    let _lock = window_lock.lock().await;
    println!("🔵 [DEBUG] [show_widget_window] 🔒 Lock acquired for {}", widget);
    
    let window_label = format!("{}-window", widget);
    
    // Hide palette if opening a different widget
    if widget != "palette" {
        if let Some(palette) = app.get_webview_window("palette-window") {
            println!("[Window] Hiding palette before opening {}", widget);
            palette.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
        }
    }
    
    // Define window dimensions
    let (width, height, _title, _transparent, _decorations) = match widget {
        "palette" => (550, 328, "Command Palette", true, false),
        "clipboard" => (500, 400, "Clipboard History", false, false),
        "translator" => (700, 550, "Translator", false, false),
        "currency" => (500, 400, "Currency Converter", false, false),
        "time_converter" => (600, 500, "Time Zone Converter", false, false),
        "settings" => (800, 600, "Settings", false, false),
        _ => (600, 400, "Widget", false, false),
    };
    
    // Check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        println!("🔵 [DEBUG] [show_widget_window] Window '{}' already exists", window_label);
        
        if widget == "palette" {
            // Reset size
            window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: width as f64,
                height: height as f64,
            })).ok();
            
            // CRITICAL: Configure and show ONCE - no verification or retry (internal retry handles it)
            #[cfg(target_os = "macos")]
            {
                system::window::nswindow::configure_window_for_fullscreen(&window).ok();
                system::window::nswindow::show_window_over_fullscreen(&window).ok();
                // That's it! Internal retry handles the rest
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true).ok();
                window.show().ok();
            }
            
            // Increased delay for full transition to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        } else {
            // CRITICAL FIX: Configure and show ALL widgets over fullscreen
            #[cfg(target_os = "macos")]
            {
                system::window::nswindow::configure_window_for_fullscreen(&window).ok();
                system::window::nswindow::show_window_over_fullscreen(&window).ok();
                // That's it! Internal retry handles the rest
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true).ok();
                window.show().ok();
            }
            
            // Increased delay for full transition to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        }
        
        println!("🔵 [DEBUG] [show_widget_window] 🔓 Lock released");
        return Ok(());
    }
    
    // Continue to new window creation...
    show_widget_window_create_new_async(app, widget, has_selection).await
}

// Sync wrapper for menu items (they can't be async)
// FIXED: Use spawn instead of block_on to avoid runtime conflicts
fn show_widget_window(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), String> {
    // Get window lock from app state
    if let Some(window_lock) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
        // Spawn async task instead of blocking
        // This avoids "Cannot start a runtime from within a runtime" panic
        let app_clone = app.clone();
        let widget_str = widget.to_string();
        let lock_clone = window_lock.inner().clone();
        
        tauri::async_runtime::spawn(async move {
            match show_widget_window_async(&app_clone, &widget_str, has_selection, lock_clone).await {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to show window '{}': {}", widget_str, e),
            }
        });
        
        // Return immediately (fire and forget for sync callers)
        Ok(())
    } else {
        // Fallback: call legacy implementation if lock not available
        show_widget_window_legacy(app, widget, has_selection)
    }
}

// Legacy implementation (kept for fallback)
fn show_widget_window_legacy(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), String> {
    let window_label = format!("{}-window", widget);
    
    // Hide palette if opening a different widget
    if widget != "palette" {
        if let Some(palette) = app.get_webview_window("palette-window") {
            println!("[Window] Hiding palette before opening {}", widget);
            palette.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
        }
    }
    
    // Define window dimensions
    let (width, height, _title, _transparent, _decorations) = match widget {
        "palette" => (550, 328, "Command Palette", true, false),
        "clipboard" => (500, 400, "Clipboard History", false, false),
        "translator" => (700, 550, "Translator", false, false),  // Increased height
        "currency" => (500, 400, "Currency Converter", false, false),  // Increased height
        "time_converter" => (600, 500, "Time Zone Converter", false, false),
        "settings" => (800, 600, "Settings", false, false),
        _ => (600, 400, "Widget", false, false),
    };
    
    // Check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        println!("🔵 [DEBUG] [show_widget_window] Window '{}' already exists", window_label);
        
        
        // For palette window, reset size to ensure stability
        println!("🔴🔴🔴 [CRITICAL DEBUG] widget = '{}', checking if == 'palette'", widget);
        if widget == "palette" {
            println!("🔴🔴🔴 [CRITICAL DEBUG] INSIDE PALETTE BLOCK - THIS SHOULD APPEAR!");
            println!("🔵 [DEBUG] [show_widget_window] Resetting palette window size to {}x{}", width, height);
            if let Err(e) = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: width as f64,
                height: height as f64,
            })) {
                eprintln!("Failed to set window size: {}", e);
            }
            
            // For existing palette window, configure for fullscreen overlay
            println!("🔵 [DEBUG] [show_widget_window] Configuring existing palette window...");
            
            // CRITICAL: Use native method to show window over fullscreen
            #[cfg(target_os = "macos")]
            {
                match system::window::nswindow::show_window_over_fullscreen(&window) {
                    Ok(_) => {
                        println!("🔵 [DEBUG] [show_widget_window] ✓ Window shown over fullscreen successfully");
                        
                        // Verify visibility
                        if let Ok((is_visible, is_on_active_space, _, _, _, _)) = system::window::nswindow::verify_window_visibility(&window) {
                            if is_visible && is_on_active_space {
                                println!("🔵 [DEBUG] [show_widget_window] ✅ Window is visible on active space!");
                            } else {
                                eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Visibility issue: visible={}, onActiveSpace={}", is_visible, is_on_active_space);
                            }
                        }
                    },
                Err(e) => {
                        eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Failed to show over fullscreen: {}", e);
                    // Fallback
                        window.set_always_on_top(true).ok();
                        window.show().ok();
                    }
                }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true).ok();
                window.show().ok();
            }
                    
                    std::thread::sleep(std::time::Duration::from_millis(100));
                            } else {
            println!("🔵 [DEBUG] [show_widget_window] Showing non-palette widget '{}'...", widget);
            
            // CRITICAL FIX: Use native method to show all widgets over fullscreen
            #[cfg(target_os = "macos")]
            {
                match system::window::nswindow::show_window_over_fullscreen(&window) {
                    Ok(_) => {
                        println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' shown over fullscreen successfully", widget);
                    },
                        Err(e) => {
                        eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Failed to show widget '{}' over fullscreen: {}", widget, e);
                    // Fallback
                        window.set_always_on_top(true).ok();
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true).ok();
                window.show().ok();
                window.set_focus().ok();
            }
        }
        
        println!("🔵 [DEBUG] [show_widget_window] Returning early (window already existed)");
        return Ok(());
    }
    
    // Create new window (sync version for legacy)
    show_widget_window_create_new_legacy(app, widget, has_selection)
}

// Create new window (async version)
async fn show_widget_window_create_new_async(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), String> {
    let window_label = format!("{}-window", widget);
    
    // Define window dimensions
    let (width, height, title, _transparent, decorations) = match widget {
        "palette" => (550, 328, "Command Palette", true, false),
        "clipboard" => (500, 400, "Clipboard History", false, false),
        "translator" => (700, 550, "Translator", false, false),
        "currency" => (500, 400, "Currency Converter", false, false),
        "time_converter" => (600, 500, "Time Zone Converter", false, false),
        "settings" => (800, 600, "Settings", false, false),
        _ => (600, 400, "Widget", false, false),
    };

    // Create new window
    let is_resizable = widget != "palette"; // Palette is non-resizable
    
    // Base builder with transparency for ALL windows initially
    // For palette, start invisible to prevent flash, then show after transparency is ready
    // CRITICAL: For palette, create window HIDDEN so we can convert to NSPanel before showing
    // Sequence: build (hidden) → convert to NSPanel → configure → THEN show()
    let start_visible = widget != "palette";
    let mut builder = WebviewWindowBuilder::new(app, &window_label, WebviewUrl::App(format!("index.html?widget={}", widget).into()))
        .title(title)
        .inner_size(width as f64, height as f64)
        .resizable(is_resizable)
        .focused(false)  // CRITICAL: Don't focus before configuration (macOS rejects it in fullscreen)
        .always_on_top(widget == "palette")  // Fallback
        .visible(start_visible)  // Palette starts hidden, other widgets can be visible
        .transparent(true)  // All windows start transparent
        .decorations(false)  // All windows start without decorations
        .skip_taskbar(true);  // Don't show in Dock/taskbar
    
    // Add decorations back for non-palette widgets
    if widget != "palette" && decorations {
        builder = builder.decorations(true).transparent(false);
    }
    
    // Add size constraints and floating properties for palette
    if widget == "palette" {
        builder = builder
            .min_inner_size(width as f64, height as f64)
            .max_inner_size(width as f64, height as f64)
            // DO NOT use visible_on_all_workspaces - we want it on CURRENT space only
            // visible_on_all_workspaces makes it appear on all spaces, which causes it to appear on wrong space
            .always_on_top(true);  // Always on top
    }
    
    // Special positioning for palette window
    if widget == "palette" {
        // CONDITIONAL POSITIONING based on selection detection
        if has_selection {
            // LOGIC A: Text was selected → Position at CURSOR
            println!("[Position] Mode: CURSOR (text selected)");
            
        use mouse_position::mouse_position::Mouse;
        
        if let Mouse::Position { x: cursor_x, y: cursor_y } = Mouse::get_mouse_position() {
            println!("[DEBUG] Raw cursor position: ({}, {})", cursor_x, cursor_y);
            
            // Get ALL monitors and find which one contains the cursor
            if let Ok(monitors) = app.available_monitors() {
                let mut target_monitor = None;
                
                // Find the monitor that contains the cursor
                for monitor in monitors {
                    let pos = monitor.position();
                    let size = monitor.size();
                    let scale = monitor.scale_factor();
                    
                    // Convert monitor bounds to physical pixels for comparison
                    let mon_x = pos.x as i32;
                    let mon_y = pos.y as i32;
                    let mon_width = (size.width as f64 * scale) as i32;
                    let mon_height = (size.height as f64 * scale) as i32;
                    
                    // Check if cursor is within this monitor's bounds (physical pixels)
                    if cursor_x >= mon_x && cursor_x < mon_x + mon_width &&
                       cursor_y >= mon_y && cursor_y < mon_y + mon_height {
                        target_monitor = Some(monitor);
                        break;
                    }
                }
                
                // Use the monitor containing the cursor, or fall back to primary
                    let monitor = target_monitor.or_else(|| app.primary_monitor().ok().flatten());
                
                if let Some(monitor) = monitor {
                    let position = monitor.position();
                    let size = monitor.size();
                    let scale_factor = monitor.scale_factor();
                    
                    let screen_x = position.x as f64;
                    let screen_y = position.y as f64;
                    let screen_width = size.width as f64;
                    let screen_height = size.height as f64;
                    
                    // Convert cursor position from physical to logical pixels
                    let cursor_logical_x = (cursor_x as f64) / scale_factor;
                    let cursor_logical_y = (cursor_y as f64) / scale_factor;
                    
                    // Calculate optimal position with boundary detection
                        const PALETTE_WIDTH: f64 = 550.0;
                    const PALETTE_HEIGHT: f64 = 328.0;
                    
                    let mut x = cursor_logical_x;
                    let mut y = cursor_logical_y;
                    
                        // Boundary checks
                    if x + PALETTE_WIDTH > screen_x + screen_width {
                        x = screen_x + screen_width - PALETTE_WIDTH;
                    }
                    if y + PALETTE_HEIGHT > screen_y + screen_height {
                        y = screen_y + screen_height - PALETTE_HEIGHT;
                        }
                        if x < screen_x { x = screen_x; }
                        if y < screen_y { y = screen_y; }
                        
                        println!("[Position] Cursor mode: ({}, {})", x, y);
                        builder = builder.position(x, y);
                    } else {
                        println!("[Position] No monitor found, using center");
                        builder = builder.center();
                    }
                } else {
                    println!("[Position] Failed to get monitors, using center");
                    builder = builder.center();
                }
            } else {
                println!("[Position] Failed to get cursor, using center");
                builder = builder.center();
            }
        } else {
            // LOGIC B: No text selected → Position at CENTER
            println!("[Position] Mode: CENTER (no selection)");
            builder = builder.center();
        }
    } else {
        // LOGIC C: All other widgets → always CENTER
        println!("[Position] Widget center positioning");
        builder = builder.center().decorations(decorations);
    }
    
    println!("🔵 [DEBUG] [show_widget_window] Building new window '{}' (hidden={})...", window_label, !start_visible);
    let window = builder.build().map_err(|e: tauri::Error| format!("Failed to build window: {}", e))?;
    println!("🔵 [DEBUG] [show_widget_window] ✓ Window built successfully");
    
        // CRITICAL FIX: Configure ALL widgets for fullscreen overlay IMMEDIATELY after creation
        // This ensures translator, currency, clipboard, settings, etc. all appear over fullscreen apps
        if widget == "palette" || widget == "translator" || widget == "currency" || widget == "clipboard" || widget == "settings" {
            println!("🔵 [DEBUG] [show_widget_window] Configuring widget '{}' for fullscreen overlay...", widget);
            
            // CRITICAL: Configure and show ONCE - no verification or retry (internal retry handles it)
            #[cfg(target_os = "macos")]
            {
                system::window::nswindow::configure_window_for_fullscreen(&window).ok();
                system::window::nswindow::show_window_over_fullscreen(&window).ok();
                // That's it! Internal retry handles the rest
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                // Non-macOS: Use standard Tauri methods
                window.set_always_on_top(true).ok();
                window.show().ok();
            }
            
            // Increased delay for full transition to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
            
            println!("🔵 [DEBUG] [show_widget_window] Widget '{}' creation complete", widget);
    }
    
    // Add blur event listener for click-outside behavior (except palette)
    if widget != "palette" {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                // Window lost focus - hide it
                let _ = window_clone.hide();
            }
        });
    }

    Ok(())
}

// Create new window (sync version for legacy fallback)
// FIXED: Use spawn instead of block_on to avoid runtime conflicts
fn show_widget_window_create_new_legacy(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), String> {
    // Spawn async task instead of blocking
    let app_clone = app.clone();
    let widget_str = widget.to_string();
    
    tauri::async_runtime::spawn(async move {
        if let Err(e) = show_widget_window_create_new_async(&app_clone, &widget_str, has_selection).await {
            eprintln!("Failed to create window '{}': {}", widget_str, e);
        }
    });
    
    // Return immediately (fire and forget)
    Ok(())
}

// NOTE: The create_floating_panel function has been removed.
// We now configure Tauri windows directly as non-activating panels
// instead of creating separate NSPanel instances.
