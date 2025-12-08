//! Window management command module
//!
//! Handles window positioning, cursor detection, and window lifecycle.

use crate::shared::types::{CursorPosition, ScreenBounds, WindowPosition};
use crate::shared::error::{AppResult, AppError};
use tauri::Manager;

/// Get the current cursor position
#[tauri::command]
pub fn get_cursor_position() -> AppResult<CursorPosition> {
    use mouse_position::mouse_position::Mouse;
    
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Ok(CursorPosition {
            x: x as f64,
            y: y as f64,
        }),
        Mouse::Error => Err(AppError::System("Failed to get cursor position".to_string())),
    }
}

/// Get the primary monitor bounds
#[tauri::command]
pub fn get_primary_monitor_bounds(app: tauri::AppHandle) -> AppResult<ScreenBounds> {
    // Get the primary monitor
    let monitor = app.primary_monitor()
        .map_err(|e| AppError::System(format!("Failed to get primary monitor: {}", e)))?
        .ok_or_else(|| AppError::System("No primary monitor found".to_string()))?;
    
    let position = monitor.position();
    let size = monitor.size();
    
    Ok(ScreenBounds {
        x: position.x as f64,
        y: position.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    })
}

/// Calculate optimal palette position based on cursor and screen bounds
#[tauri::command]
pub fn calculate_palette_position(
    cursor_x: f64,
    cursor_y: f64,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
) -> WindowPosition {
    const PALETTE_WIDTH: f64 = 270.0;
    const PALETTE_HEIGHT: f64 = 328.0;
    const POPOVER_WIDTH: f64 = 280.0; // Space for popover on the right
    const TOTAL_WIDTH: f64 = PALETTE_WIDTH + POPOVER_WIDTH; // 550px total
    
    let mut x = cursor_x;
    let mut y = cursor_y;
    
    // Check right boundary (palette + popover)
    if x + TOTAL_WIDTH > screen_x + screen_width {
        x = screen_x + screen_width - TOTAL_WIDTH;
    }
    
    // Check bottom boundary
    if y + PALETTE_HEIGHT > screen_y + screen_height {
        y = screen_y + screen_height - PALETTE_HEIGHT;
    }
    
    // Check left boundary
    if x < screen_x {
        x = screen_x;
    }
    
    // Check top boundary
    if y < screen_y {
        y = screen_y;
    }
    
    WindowPosition { x, y }
}

/// Hide the palette window
#[tauri::command]
pub async fn hide_palette_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("palette-window") {
        window.hide().map_err(|e| AppError::System(format!("Failed to hide palette: {}", e)))?;
    }
    Ok(())
}

// NOTE: focus_palette_window has been removed.
// Non-activating panels should NOT be focused as it activates the app.
// Use order_window_front() instead if you need to bring the window to front.

/// Show a widget window
#[tauri::command]
pub async fn show_widget(app: tauri::AppHandle, widget: String) -> AppResult<()> {
    // FIXED: Call async version directly instead of sync wrapper
    if let Some(window_lock) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
        crate::show_widget_window_async(&app, &widget, false, window_lock.inner().clone()).await
            .map_err(|e| AppError::System(format!("Failed to show {} widget: {}", widget, e)))
    } else {
        Err(AppError::System("Window lock not available".to_string()))
    }
}
