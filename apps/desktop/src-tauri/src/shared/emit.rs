use tauri::{AppHandle, Manager, Emitter};
use super::events::AppEvent;

/// Emit an application event to all windows
pub fn emit_event(app: &AppHandle, event: AppEvent) {
    // We need to manually match and emit because Tauri's emit function takes a string event name
    // The AppEvent enum encapsulates both the name (via serde rename) and payload
    // However, for simplicity in Rust->Tauri bridge, we'll use a standardized wrapper or specific calls.
    
    // Strategy: Use the serde-serialized name as the event name
    // This requires a bit of manual mapping or a trait.
    // For now, let's implement a simple match dispatch.
    
    match &event {
        AppEvent::ClipboardUpdated(item) => {
            if let Err(e) = app.emit("clipboard://updated", item) {
                eprintln!("Failed to emit clipboard update: {}", e);
            }
        }

        AppEvent::SettingsUpdated(settings) => {
            if let Err(e) = app.emit("settings://update", settings) {
                eprintln!("Failed to emit settings update: {}", e);
            }
        }
        AppEvent::WindowFocusChanged(focused) => {
            if let Err(e) = app.emit("window://focus", focused) {
                eprintln!("Failed to emit window focus: {}", e);
            }
        }
    }
}
