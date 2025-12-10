use crate::shared::types::ClipboardHistoryItem;
use crate::shared::events::AppEvent;
use crate::shared::emit::emit_event;
use super::history::ClipboardHistory;
use std::sync::{Arc, Mutex}; // Keep Mutex as it's used
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, Emitter};
use tokio::time::{sleep, Duration}; // Use tokio's Duration and sleep
use chrono::Local;
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
            
            let mut consecutive_errors = 0u32;
            const MAX_CONSECUTIVE_ERRORS: u32 = 10;
            const BASE_POLL_INTERVAL_MS: u64 = 500;
            const MAX_POLL_INTERVAL_MS: u64 = 5000;

            loop {
                // Check if monitoring is enabled (with mutex recovery)
                let is_enabled = match enabled.lock() {
                    Ok(guard) => *guard,
                    Err(poisoned) => {
                        eprintln!("[ClipboardMonitor] Mutex poisoned, recovering...");
                        *poisoned.into_inner()
                    }
                };
                if !is_enabled {
                    tokio::time::sleep(Duration::from_millis(BASE_POLL_INTERVAL_MS)).await;
                    consecutive_errors = 0; // Reset error count when disabled
                    continue;
                }

                // Calculate sleep interval based on clipboard read result
                // This ensures backoff is properly applied
                let sleep_interval = match app.clipboard().read_text() {
                    Ok(current_content) => {
                        // Reset error counter on successful read
                        consecutive_errors = 0;
                        
                        if current_content.is_empty() {
                            BASE_POLL_INTERVAL_MS
                        } else {
                            // Check for "Ghost Copy" flag
                            let clipboard_state = app.state::<crate::core::clipboard::ClipboardState>();
                            let should_ignore = clipboard_state.ignore_next.swap(false, Ordering::SeqCst);

                            // 1. Check if content has changed (Cheap check)
                            let has_changed = {
                                let last = match last_content.lock() {
                                    Ok(guard) => guard,
                                    Err(poisoned) => poisoned.into_inner(),
                                };
                                match &*last {
                                    Some(prev) => prev != &current_content,
                                    None => true,
                                }
                            };

                            if should_ignore {
                                if has_changed {
                                    println!("[ClipboardMonitor] 👻 Ghost copy detected and ignored.");
                                    // Update last_content state so we don't process this as a new change later
                                    {
                                        let mut last = match last_content.lock() {
                                            Ok(guard) => guard,
                                            Err(poisoned) => poisoned.into_inner(),
                                        };
                                        *last = Some(current_content.clone());
                                    }
                                } else {
                                    println!("[ClipboardMonitor] 👻 Ghost flag consumed but content unchanged.");
                                }
                                BASE_POLL_INTERVAL_MS
                            } else if !has_changed {
                                BASE_POLL_INTERVAL_MS
                            } else {
                                println!("[ClipboardMonitor] Detected clipboard change");

                                // 2. Heavy operations (only if changed)
                                let active_app = crate::system::automation::macos::get_active_app().ok();
                                let current_content_string = current_content.clone();

                                // 3. Update last_content state (to prevent re-processing)
                                {
                                    let mut last = match last_content.lock() {
                                        Ok(guard) => guard,
                                        Err(poisoned) => poisoned.into_inner(),
                                    };
                                    *last = Some(current_content.clone());
                                }

                                // 4. Check sensitivity
                                if crate::core::clipboard::filter::is_sensitive(&current_content_string, active_app.as_deref()) {
                                    println!("[ClipboardMonitor] 🔒 Sensitive content detected. Ignoring.");
                                    BASE_POLL_INTERVAL_MS
                                } else {
                                    // 5. Add to history
                                    let item = crate::shared::types::ClipboardHistoryItem::new_text(
                                        current_content_string.clone(), 
                                        active_app.clone()
                                    );
                                    
                                    history.add_item(item.clone());
                                    
                                    emit_event(&app, AppEvent::ClipboardUpdated(item));
                                    
                                    println!("✅ Clipboard updated: \"{}\"", 
                                        if current_content_string.len() > 20 { 
                                            format!("{}...", &current_content_string[0..20]) 
                                        } else { 
                                            current_content_string.clone() 
                                        }
                                    );
                                    
                                    BASE_POLL_INTERVAL_MS
                                }
                            }
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        
                        // Only log errors occasionally to avoid spam
                        if consecutive_errors == 1 || consecutive_errors % 10 == 0 {
                            eprintln!("[ClipboardMonitor] Failed to read clipboard (error #{}) : {}", consecutive_errors, e);
                        }
                        
                        // Calculate backoff interval based on error count
                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            if consecutive_errors == MAX_CONSECUTIVE_ERRORS {
                                eprintln!("[ClipboardMonitor] ⚠️  Too many consecutive errors. Reducing polling frequency.");
                            }
                            
                            // Exponential backoff up to MAX_POLL_INTERVAL_MS
                            std::cmp::min(
                                BASE_POLL_INTERVAL_MS * (2_u64.pow((consecutive_errors - MAX_CONSECUTIVE_ERRORS).min(4))),
                                MAX_POLL_INTERVAL_MS
                            )
                        } else {
                            BASE_POLL_INTERVAL_MS
                        }
                    }
                };

                // Single sleep point that respects backoff
                let sleep_duration = Duration::from_millis(sleep_interval);
                tokio::time::sleep(sleep_duration).await;
            }
        });
    }


    /// Enable clipboard monitoring
    pub fn enable(&self) {
        match self.enabled.lock() {
            Ok(mut enabled) => {
                *enabled = true;
                println!("[ClipboardMonitor] Enabled");
            }
            Err(poisoned) => {
                eprintln!("[ClipboardMonitor] Mutex poisoned in enable(), recovering...");
                let mut guard = poisoned.into_inner();
                *guard = true;
                println!("[ClipboardMonitor] Enabled (after recovery)");
            }
        }
    }

    /// Disable clipboard monitoring
    pub fn disable(&self) {
        match self.enabled.lock() {
            Ok(mut enabled) => {
                *enabled = false;
                println!("[ClipboardMonitor] Disabled");
            }
            Err(poisoned) => {
                eprintln!("[ClipboardMonitor] Mutex poisoned in disable(), recovering...");
                let mut guard = poisoned.into_inner();
                *guard = false;
                println!("[ClipboardMonitor] Disabled (after recovery)");
            }
        }
    }

    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        match self.enabled.lock() {
            Ok(enabled) => *enabled,
            Err(poisoned) => {
                eprintln!("[ClipboardMonitor] Mutex poisoned in is_enabled(), recovering...");
                *poisoned.into_inner()
            }
        }
    }

    /// Toggle monitoring on/off
    pub fn toggle(&self) -> bool {
        match self.enabled.lock() {
            Ok(mut enabled) => {
                *enabled = !*enabled;
                let new_state = *enabled;
                println!("[ClipboardMonitor] Toggled to {}", new_state);
                new_state
            }
            Err(poisoned) => {
                eprintln!("[ClipboardMonitor] Mutex poisoned in toggle(), recovering...");
                let mut guard = poisoned.into_inner();
                *guard = !*guard;
                let new_state = *guard;
                println!("[ClipboardMonitor] Toggled to {} (after recovery)", new_state);
                new_state
            }
        }
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
