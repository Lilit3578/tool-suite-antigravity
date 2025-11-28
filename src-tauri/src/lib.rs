mod settings;
mod types;
mod commands;
mod clipboard;
mod automation;
mod context;

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
            // Load settings
            let _settings = settings::AppSettings::load()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to load settings: {}", e);
                    settings::AppSettings::default()
                });

            // Initialize clipboard history and monitor
            let clipboard_history = clipboard::ClipboardHistory::new();
            let clipboard_monitor = clipboard::ClipboardMonitor::new(clipboard_history.clone_arc());
            
            // Initialize usage metrics for intelligent ranking
            let usage_metrics = context::UsageMetrics::new();
            
            // Initialize last active app tracker
            let last_active_app = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
            
            // Store in app state for access from commands
            app.manage(clipboard_history);
            app.manage(clipboard_monitor.clone_arc());
            app.manage(usage_metrics);
            app.manage(last_active_app);
            
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
                            if let Err(e) = show_widget_window(app, "palette") {
                                eprintln!("Failed to show palette: {}", e);
                            }
                        }
                        "clipboard" => {
                            if let Err(e) = show_widget_window(app, "palette") {
                                eprintln!("Failed to show clipboard history: {}", e);
                            }
                        }
                        "toggle_monitor" => {
                            if let Some(monitor) = app.try_state::<clipboard::ClipboardMonitor>() {
                                let enabled = monitor.toggle();
                                println!("Clipboard monitoring: {}", if enabled { "enabled" } else { "disabled" });
                                
                                // Update menu item text
                                // Note: Tauri doesn't support dynamic menu text updates easily
                                // This would require rebuilding the tray menu
                            }
                        }
                        "clear_history" => {
                            if let Some(history) = app.try_state::<clipboard::ClipboardHistory>() {
                                history.clear();
                                println!("Clipboard history cleared");
                            }
                        }
                        "settings" => {
                            if let Err(e) = show_widget_window(app, "settings") {
                                eprintln!("Failed to show settings: {}", e);
                            }
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
            let shortcut_str = "Control+Shift+L";
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
                    
                    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                        // Spawn async task to avoid blocking the shortcut handler
                        let handle_clone = handle.clone();
                        let last_app_clone = last_app_clone.clone();
                        
                        tauri::async_runtime::spawn(async move {
                            // Capture active app with proper error handling and mutex recovery
                            match automation::get_active_app() {
                                Ok(active_app) => {
                                    match last_app_clone.lock() {
                                        Ok(mut last_app) => {
                                            *last_app = Some(active_app.clone());
                                            println!("[Shortcut] Stored last active app: {}", active_app);
                                        }
                                        Err(poisoned) => {
                                            eprintln!("[Shortcut] Mutex poisoned, recovering...");
                                            let mut guard = poisoned.into_inner();
                                            *guard = Some(active_app.clone());
                                            println!("[Shortcut] Stored last active app: {} (after recovery)", active_app);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[Shortcut] Failed to get active app: {}", e);
                                    // Continue anyway - might still work
                                }
                            }
                            
                            // Simulate Cmd+C with permission check
                            if let Err(e) = automation::simulate_cmd_c() {
                                eprintln!("[Shortcut] Failed to simulate Cmd+C: {}", e);
                                return; // Don't open palette if copy failed
                            }
                            
                            // Async sleep instead of blocking
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            
                            // Open palette window
                            if let Err(e) = show_widget_window(&handle_clone, "palette") {
                                eprintln!("[Shortcut] Failed to show palette window: {}", e);
                            } else {
                                // Explicitly focus the palette window
                                if let Some(window) = handle_clone.get_webview_window("palette-window") {
                                    let _ = window.set_focus();
                                }
                            }
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
            println!("📋 Global Shortcut: Control+Shift+L");
            println!("💡 All widgets and actions accessible via Command Palette");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::capture_selection,
            commands::translate_text,
            commands::convert_currency,
            commands::get_settings,
            commands::save_settings,
            commands::log_message,
            commands::get_command_items,
            commands::execute_action,
            commands::get_cursor_position,
            commands::get_primary_monitor_bounds,
            commands::calculate_palette_position,
            commands::hide_palette_window,
            commands::focus_palette_window,
            commands::get_clipboard_history,
            commands::paste_clipboard_item,
            commands::clear_clipboard_history,
            commands::toggle_clipboard_monitor,
            commands::get_clipboard_monitor_status,
            commands::get_active_app,
            commands::check_accessibility_permissions,
            commands::record_command_usage,
            commands::show_widget,
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

fn show_widget_window(app: &tauri::AppHandle, widget: &str) -> Result<(), Box<dyn std::error::Error>> {
    let window_label = format!("{}-window", widget);
    
    // Define window dimensions
    let (width, height, title, _transparent, decorations) = match widget {
        "palette" => (550, 328, "Command Palette", true, false),
        "clipboard" => (500, 400, "Clipboard History", false, false),
        "translator" => (700, 550, "Translator", false, false),  // Increased height
        "currency" => (500, 400, "Currency Converter", false, false),  // Increased height
        "settings" => (800, 600, "Settings", false, false),
        _ => (600, 400, "Widget", false, false),
    };
    
    // Check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        // For palette window, reset size to ensure stability
        if widget == "palette" {
            println!("[DEBUG] Resetting palette window size to {}x{}", width, height);
            window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: width as f64,
                height: height as f64,
            }))?;
            // Ensure window is hidden until transparency is ready
            window.hide()?;
            // Small delay to ensure transparency is applied
            std::thread::sleep(std::time::Duration::from_millis(10));
            window.show()?;
        } else {
            window.show()?;
        }
        // Ensure focus is set (double-focus for palette to ensure it sticks)
        window.set_focus()?;
        if widget == "palette" {
            std::thread::sleep(std::time::Duration::from_millis(50));
            window.set_focus()?;
        }
        return Ok(());
    }

    // Create new window
    let is_resizable = widget != "palette"; // Palette is non-resizable
    
    // Base builder with transparency for ALL windows initially
    // For palette, start invisible to prevent flash, then show after transparency is ready
    let start_visible = widget != "palette";
    let mut builder = WebviewWindowBuilder::new(app, &window_label, WebviewUrl::App(format!("index.html?widget={}", widget).into()))
        .title(title)
        .inner_size(width as f64, height as f64)
        .resizable(is_resizable)
        .focused(true)
        .always_on_top(true)
        .visible(start_visible)
        .transparent(true)  // All windows start transparent
        .decorations(false)  // All windows start without decorations
        .skip_taskbar(true);  // Don't show in Dock/taskbar
    
    // Add decorations back for non-palette widgets
    if widget != "palette" && decorations {
        builder = builder.decorations(true).transparent(false);
    }
    
    // Add size constraints for palette to prevent resizing
    if widget == "palette" {
        builder = builder
            .min_inner_size(width as f64, height as f64)
            .max_inner_size(width as f64, height as f64);
    }
    
    // Special positioning for palette window
    if widget == "palette" {
        // Get cursor position and screen bounds
        use mouse_position::mouse_position::Mouse;
        
        println!("[DEBUG] Attempting to get cursor position...");
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
                    
                    println!("[DEBUG] Checking monitor: pos=({}, {}), size={}x{} (physical), scale={}", 
                        mon_x, mon_y, mon_width, mon_height, scale);
                    
                    // Check if cursor is within this monitor's bounds (physical pixels)
                    if cursor_x >= mon_x && cursor_x < mon_x + mon_width &&
                       cursor_y >= mon_y && cursor_y < mon_y + mon_height {
                        target_monitor = Some(monitor);
                        println!("[DEBUG] ✓ Cursor is on this monitor!");
                        break;
                    }
                }
                
                // Use the monitor containing the cursor, or fall back to primary
                let monitor = target_monitor.or_else(|| {
                    println!("[DEBUG] Cursor not found on any monitor, using primary");
                    app.primary_monitor().ok().flatten()
                });
                
                if let Some(monitor) = monitor {
                    let position = monitor.position();
                    let size = monitor.size();
                    let scale_factor = monitor.scale_factor();
                    
                    let screen_x = position.x as f64;
                    let screen_y = position.y as f64;
                    let screen_width = size.width as f64;
                    let screen_height = size.height as f64;
                    
                    println!("[DEBUG] Using monitor: x={}, y={}, width={}, height={} (logical)", 
                        screen_x, screen_y, screen_width, screen_height);
                    println!("[DEBUG] Scale factor: {}", scale_factor);
                    
                    // Convert cursor position from physical to logical pixels
                    let cursor_logical_x = (cursor_x as f64) / scale_factor;
                    let cursor_logical_y = (cursor_y as f64) / scale_factor;
                    
                    println!("[DEBUG] Cursor position (logical): ({}, {})", cursor_logical_x, cursor_logical_y);
                    
                    // Calculate optimal position with boundary detection
                    const PALETTE_WIDTH: f64 = 550.0; // Full window width
                    const PALETTE_HEIGHT: f64 = 328.0;
                    
                    let mut x = cursor_logical_x;
                    let mut y = cursor_logical_y;
                    
                    println!("[DEBUG] Initial position: ({}, {})", x, y);
                    
                    // Check right boundary
                    if x + PALETTE_WIDTH > screen_x + screen_width {
                        x = screen_x + screen_width - PALETTE_WIDTH;
                        println!("[DEBUG] Adjusted for right boundary: x={}", x);
                    }
                    
                    // Check bottom boundary
                    if y + PALETTE_HEIGHT > screen_y + screen_height {
                        y = screen_y + screen_height - PALETTE_HEIGHT;
                        println!("[DEBUG] Adjusted for bottom boundary: y={}", y);
                    }
                    
                    // Check left boundary
                    if x < screen_x {
                        x = screen_x;
                        println!("[DEBUG] Adjusted for left boundary: x={}", x);
                    }
                    
                    // Check top boundary
                    if y < screen_y {
                        y = screen_y;
                        println!("[DEBUG] Adjusted for top boundary: y={}", y);
                    }
                    
                    println!("[DEBUG] Final position: ({}, {})", x, y);
                    
                    // Position at calculated coordinates
                    builder = builder.position(x, y);
                } else {
                    println!("[DEBUG] No monitor found, using center");
                    builder = builder.center();
                }
            } else {
                println!("[DEBUG] Failed to get monitors, using center");
                builder = builder.center();
            }
        } else {
            println!("[DEBUG] Failed to get cursor position, using center");
            // Fallback to center if cursor position unavailable
            builder = builder.center();
        }
        // Palette transparency already set in base builder
    } else {
        // Other widgets use center positioning
        builder = builder.center().decorations(decorations);
    }
    
    let window = builder.build()?;
    
    // For palette window, ensure transparency is ready before showing
    if widget == "palette" {
        // Small delay to ensure transparency is applied
        std::thread::sleep(std::time::Duration::from_millis(10));
        window.show()?;
        // Ensure focus is set (double-focus to ensure it sticks)
        window.set_focus()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        window.set_focus()?;
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
