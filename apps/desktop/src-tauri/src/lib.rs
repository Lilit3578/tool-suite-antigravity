// Module declarations - these are re-exported from their respective modules
mod shared;
mod api;
mod core;
mod system;
mod config;
// mod features;  <-- REMOVED

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder, Emitter,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                     let urls = event.urls();
                     println!("Deep Link URL: {:?}", urls);
                     // urls is usually Vec<Url>
                     
                     for url in urls {
                         if url.scheme() == "prodwidgets" {
                             // 1. Parse Token
                             if let Some(token_pair) = url.query_pairs().find(|(key, _)| key == "token") {
                                 let token = token_pair.1.to_string();
                                 println!("Deep Link Token Found: {}", token);
                                 
                                 // 2. Focus Window
                                 let handle_clone = handle.clone();
                                 let token_clone = token.clone();
                                 
                                 tauri::async_runtime::spawn(async move {
                                     // Force app to front
                                     if let Some(window) = handle_clone.get_webview_window("palette-window") {
                                         system::window::nswindow::force_window_to_front(&window);
                                         window.set_focus().ok();
                                         
                                          // 3. Emit Event
                                         println!("Emitting auth-deep-link event...");
                                         if let Err(e) = handle_clone.emit("auth-deep-link", token_clone) {
                                             eprintln!("Failed to emit auth event: {}", e);
                                         }
                                     }
                                 });
                             }
                         }
                     }
                });
            }

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
            // Load settings
            // Load settings asynchronously to avoid blocking main thread
            tauri::async_runtime::spawn(async {
                if let Err(e) = shared::settings::AppSettings::load().await {
                    eprintln!("Failed to load settings: {}", e);
                } else {
                    println!("✅ Settings loaded");
                }
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

            // Initialize clipboard state (for ignoring ghost copies)
            let clipboard_state = core::clipboard::ClipboardState::new();
            
            // Store in app state for access from commands
            // CRITICAL: Must manage ALL state BEFORE accessing it in closures
            app.manage(clipboard_history);
            app.manage(clipboard_monitor.clone_arc());
            app.manage(usage_metrics);
            app.manage(last_active_app);
            app.manage(window_lock);
            app.manage(shortcut_debounce.clone());
            app.manage(clipboard_state);
            


            // Start clipboard monitoring
            clipboard_monitor.start(app.handle().clone());
            println!("✅ Clipboard monitoring started");

            // Create tray menu (Command Palette as single access point)
            let palette_item = MenuItem::with_id(app, "palette", "Open Command Palette", true, None::<&str>)?;

            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &palette_item,
                    &separator,

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
                    
                    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                        // FIXED: Only trigger on KeyPress, ignore KeyRelease
                        if event.state() == ShortcutState::Pressed {
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
                                
                                // Get clipboard state to pass ignore flag
                                let clipboard_state = handle_clone.state::<core::clipboard::ClipboardState>();
                                let ignore_flag = Some(clipboard_state.ignore_next.clone());

                                let (has_selection, _selected_text) = match system::automation::macos::detect_text_selection(&handle_clone, ignore_flag).await {
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
                                                // Optimized: Reduced from 400ms to 50ms - just enough for frame alignment
                                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                                
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
                                            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                                            window.set_focus().ok();
                                        }
                                    }
                                }
                                
                                // DONE: Window is shown and has focus, ready for user input
                                println!("🔵 [DEBUG] [Shortcut] ✅ Window shown with focus - ready for typing!");
                                
                                println!("🔵 [DEBUG] ========== SHORTCUT HANDLER COMPLETE ==========");
                            });
                        }
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
            api::commands::palette::get_command_index,
            api::commands::palette::get_command_items,
            api::commands::palette::execute_action,
            api::commands::palette::record_command_usage,
            // Window commands
            api::commands::window::get_cursor_position,
            api::commands::window::get_primary_monitor_bounds,
            api::commands::window::calculate_palette_position,
            api::commands::window::hide_palette_window,
            api::commands::window::show_widget,
            api::commands::window::debug_window_count,
            // System commands
            api::commands::system::get_active_app,
            api::commands::system::check_accessibility_permissions,
            api::commands::system::check_accessibility_permission,
            api::commands::system::log_message,
            // Settings commands
            api::commands::settings::get_settings,
            api::commands::settings::save_settings,
            // Feature commands
            // Feature commands
            core::features::translator::translate_text,
            core::features::currency::convert_currency,
            core::features::clipboard::get_clipboard_history,
            core::features::clipboard::paste_clipboard_item,

            core::features::time_converter::convert_time,
            core::features::time_converter::get_timezones,
            core::features::time_converter::parse_time_from_selection,
            core::features::time_converter::get_system_timezone,
            core::features::definition::lookup_definition,
            core::features::text_analyser::analyze_text,

            // Unit Converter commands (new registry-based API)
            core::features::unit_converter::parse_text_command,
            core::features::unit_converter::get_all_units_command,
            core::features::unit_converter::convert_units_command,
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
    
    // Hide palette logic MOVED to after show to prevent deactivation
    // if widget != "palette" { ... } 
    
    // Get window configuration from registry (no hardcoded tuples)
    let config = config::get_window_config(widget);
    let _width = config.width as u32;
    let _height = config.height as u32;
    let _title = config.title;
    let _transparent = config.transparent;
    let _decorations = config.decorations;
    
    // SINGLETON PATTERN: O(1) lookup - check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        println!("♻️ [Singleton] Reusing existing widget: {}", window_label);
        
        // IDEMPOTENT OPERATION: Simply show and focus - no re-initialization
        // Window is already configured from initial creation, so we just toggle visibility
        
        // CRITICAL: Explicitly show the window first (it might be hidden from previous close)
        window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        
        // DEBUG: Force DevTools to open so we can see Console Errors
        // window.open_devtools();
        
        // FIX: Use force_window_to_front instead of standard show/focus
        // This ensures the app activates even in Accessory mode
        system::window::nswindow::force_window_to_front(&window);
        
        // For non-palette widgets, center on show (idempotent)
        if widget != "palette" {
            window.center().ok();
        }
        
        // CRITICAL FIX: Hide palette AFTER showing the new window
        // This prevents the "zero windows visible" state which causes the OS to deactivate the app
        if widget != "palette" {
            if let Some(palette) = app.get_webview_window("palette-window") {
                println!("[Window] Hiding palette AFTER opening {}", widget);
                // Delay slightly to ensure new window has taken focus? 
                // No, standard execution order should be fine because force_window_to_front activates the app
                palette.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
            }
        }
        
        println!("🔵 [DEBUG] [show_widget_window] 🔓 Lock released");
        return Ok(());
    }
    
    // Continue to new window creation...
    show_widget_window_create_new_async(app, widget, has_selection).await?;
    
    // CRITICAL FIX FOR NEW WINDOWS TOO: Hide palette AFTER showing the new window
    // This logic must be here to cover both Reuse (above) and Create New (here) paths check?
    // Actually, I put the logic inside the Reuse block. I need to replicate it here or move it out.
    // Better to move it out to a common place at the end, but the Reuse block returns early.
    // So I will just add it here for the Create New path.
    
    if widget != "palette" {
        if let Some(palette) = app.get_webview_window("palette-window") {
            println!("[Window] Hiding palette AFTER creating/showing {}", widget);
            palette.hide().map_err(|e| format!("Failed to hide palette: {}", e))?;
        }
    }
    
    Ok(())
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
        // Fallback: spawn async task without lock
        let app_clone = app.clone();
        let widget_str = widget.to_string();
        
        tauri::async_runtime::spawn(async move {
            // Create a temporary lock for this operation
            let temp_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
            match show_widget_window_async(&app_clone, &widget_str, has_selection, temp_lock).await {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to show window '{}': {}", widget_str, e),
            }
        });
        
        Ok(())
    }
}



// Create new window (async version)
async fn show_widget_window_create_new_async(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), String> {
    let window_label = format!("{}-window", widget);
    
    // Define window dimensions
    // Define window dimensions using centralized config
    let config = config::get_window_config(widget);
    let width = config.width;
    let height = config.height;
    let title = &config.title;
    
    // Legacy hardcoded match removed - now using config module exclusively

    // Create new window
    let is_resizable = config.resizable;
    let decorations = config.decorations;
    
    // Base builder with transparency for ALL windows initially
    // For palette, start invisible to prevent flash, then show after transparency is ready
    // CRITICAL: For palette, create window HIDDEN so we can convert to NSPanel before showing
    // Sequence: build (hidden) → convert to NSPanel → configure → THEN show()
    // UPDATE: Now we do this for ALL widgets to support fullscreen overlay
    let start_visible = false;
    let mut builder = WebviewWindowBuilder::new(app, &window_label, WebviewUrl::App(format!("index.html?widget={}", widget).into()))
        .title(title)
        .inner_size(width as f64, height as f64)
        .resizable(is_resizable)
        .focused(false)  // CRITICAL: Don't focus before configuration (macOS rejects it in fullscreen)
        .always_on_top(widget == "palette")  // Fallback
        .visible(start_visible)  // Start hidden to allow configuration
        .transparent(true)  // All windows start transparent
        .decorations(false)  // All windows start without decorations
        .skip_taskbar(true);  // Don't show in Dock/taskbar
    
    // Add decorations back for non-palette widgets if configured
    if widget != "palette" && decorations {
        builder = builder.decorations(true).transparent(false).hidden_title(true);
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
        // NOTE: This configuration happens ONCE during creation - singleton reuse skips this
        if widget == "palette" || widget == "translator" || widget == "currency" || widget == "clipboard" || widget == "settings" || widget == "time_converter" || widget == "unit_converter" || widget == "definition" || widget == "text_analyser" {
            println!("🔵 [DEBUG] [show_widget_window] Configuring widget '{}' for fullscreen overlay...", widget);
            
            // CRITICAL: Configure using handle manager (only during creation)
            #[cfg(target_os = "macos")]
            {
                if let Some(handle_manager) = app.try_state::<system::window::handle::WindowHandleManager>() {
                    // Register handle first
                    if let Err(e) = system::window::nswindow::register_window_handle(&window, &window_label, handle_manager.inner()) {
                        eprintln!("⚠️ [WindowHandle] Failed to register handle: {}", e);
                    }
                    
                    // Configure using handle manager
                    if let Err(e) = system::window::nswindow::configure_for_fullscreen_overlay(&window, &window_label, handle_manager.inner()) {
                        eprintln!("⚠️ [Fullscreen] Configuration failed: {}", e);
                    }
                    
                    // Show using handle manager
                    if let Err(e) = system::window::nswindow::show_window_over_fullscreen_with_handle(&window, &window_label, handle_manager.inner()) {
                        eprintln!("⚠️ [Fullscreen] Failed to show window: {}", e);
                    }
                }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                // Non-macOS: Use standard Tauri methods
                window.set_always_on_top(true).ok();
                window.show().ok();
            }
            
        // DEBUG: Force DevTools to open so we can see Console Errors
        // window.open_devtools();
            
            // Optimized: Reduced from 400ms to 50ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            
            // CRITICAL FIX: Force window to front after creation/configuration
            // This ensures the window is actually key and active, preventing "Double Press" bug
            println!("🔵 [DEBUG] [show_widget_window] Forcing new window '{}' to front...", widget);
            system::window::nswindow::force_window_to_front(&window);
            
            println!("🔵 [DEBUG] [show_widget_window] Widget '{}' creation complete", widget);
    }
    
    // Fallback for widgets NOT in the whitelist (if any)
    // We should probably force them too if they are just created?
    // But currently all widgets seem to be in the whitelist.
    
    // SINGLETON PATTERN: Register event listeners ONCE during creation only
    // This prevents the closure stack overflow leak from duplicate registrations
    let window_clone = window.clone();
    let widget_clone = widget.to_string();
    
    window.on_window_event(move |event| {
        match event {
            // Hide window on blur (except palette)
            tauri::WindowEvent::Focused(false) if widget_clone != "palette" => {
                let _ = window_clone.hide();
            }
            
            // THE FIX: Never destroy, always hide (O(1) toggle, not O(N) re-init)
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close(); // Prevent actual destruction
                let _ = window_clone.hide(); // Hide instead (preserves singleton)
                println!("🔵 [Singleton] Window '{}' hidden (not destroyed) - ready for reuse", widget_clone);
            }
            
            _ => {}
        }
    });

    Ok(())
}



// NOTE: The create_floating_panel function has been removed.
// We now configure Tauri windows directly as non-activating panels
// instead of creating separate NSPanel instances.
