use super::{ClipboardHistory, ClipboardItem};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Clipboard monitor that polls for changes
pub struct ClipboardMonitor {
    enabled: Arc<Mutex<bool>>,
    last_content: Arc<Mutex<Option<String>>>,
    history: ClipboardHistory,
}

impl ClipboardMonitor {
    /// Create a new clipboard monitor
    pub fn new(history: ClipboardHistory) -> Self {
        Self {
            enabled: Arc::new(Mutex::new(true)),
            last_content: Arc::new(Mutex::new(None)),
            history,
        }
    }

    /// Start monitoring clipboard changes
    pub fn start(&self, app: AppHandle) {
        let enabled = Arc::clone(&self.enabled);
        let last_content = Arc::clone(&self.last_content);
        let history = self.history.clone_arc();

        // Spawn background task
        tauri::async_runtime::spawn(async move {
            println!("[ClipboardMonitor] Started monitoring");

            loop {
                // Check if monitoring is enabled
                let is_enabled = *enabled.lock().unwrap();
                if !is_enabled {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }

                // Read current clipboard content
                match app.clipboard().read_text() {
                    Ok(current_content) => {
                        if current_content.is_empty() {
                            // Skip empty clipboard
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            continue;
                        }

                        // Check if content has changed
                        let mut last = last_content.lock().unwrap();
                        let has_changed = match &*last {
                            Some(prev) => prev != &current_content,
                            None => true,
                        };

                        if has_changed {
                            println!("[ClipboardMonitor] Detected clipboard change");

                            // Update last content
                            *last = Some(current_content.clone());
                            drop(last);

                            // Get the active app (source of the clipboard content)
                            let source_app = crate::automation::get_active_app().ok();
                            
                            // Add to history
                            let item = ClipboardItem::new_text(current_content, source_app);
                            history.add_item(item);

                            // Emit event to frontend
                            if let Err(e) = app.emit("clipboard-changed", history.get_items()) {
                                eprintln!("[ClipboardMonitor] Failed to emit event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ClipboardMonitor] Failed to read clipboard: {}", e);
                    }
                }

                // Poll every 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    /// Enable clipboard monitoring
    pub fn enable(&self) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = true;
        println!("[ClipboardMonitor] Enabled");
    }

    /// Disable clipboard monitoring
    pub fn disable(&self) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = false;
        println!("[ClipboardMonitor] Disabled");
    }

    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// Toggle monitoring on/off
    pub fn toggle(&self) -> bool {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = !*enabled;
        let new_state = *enabled;
        println!("[ClipboardMonitor] Toggled to {}", new_state);
        new_state
    }

    /// Get a clone for sharing across threads
    pub fn clone_arc(&self) -> Self {
        Self {
            enabled: Arc::clone(&self.enabled),
            last_content: Arc::clone(&self.last_content),
            history: self.history.clone_arc(),
        }
    }
}
