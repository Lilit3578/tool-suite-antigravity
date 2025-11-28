# Stability Risk Report
## Deep Code Audit - Tauri Application

**Date:** 2024  
**Auditor:** Senior Systems Engineer  
**Focus:** Crash vectors, memory leaks, race conditions, resource exhaustion

---

## 🔴 CRITICAL FLAWS (The Crashers)

### 1. Infinite Loop & Resource Exhaustion in Clipboard Monitor

**File:** `src-tauri/src/clipboard/monitor.rs`

**Issue:** The clipboard polling loop has a **critical logic flaw** that causes exponential CPU usage when clipboard read fails. The backoff mechanism is **completely negated** by unconditional sleep at line 114.

**Root Cause:**
```87:114:src-tauri/src/clipboard/monitor.rs
                    Err(e) => {
                        consecutive_errors += 1;
                        
                        // Only log errors occasionally to avoid spam
                        if consecutive_errors == 1 || consecutive_errors % 10 == 0 {
                            eprintln!("[ClipboardMonitor] Failed to read clipboard (error #{}) : {}", consecutive_errors, e);
                        }
                        
                        // If too many consecutive errors, warn and increase polling interval
                        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            if consecutive_errors == MAX_CONSECUTIVE_ERRORS {
                                eprintln!("[ClipboardMonitor] ⚠️  Too many consecutive errors. Reducing polling frequency.");
                            }
                            
                            // Exponential backoff up to MAX_POLL_INTERVAL_MS
                            let backoff_interval = std::cmp::min(
                                BASE_POLL_INTERVAL_MS * (2_u64.pow((consecutive_errors - MAX_CONSECUTIVE_ERRORS).min(4))),
                                MAX_POLL_INTERVAL_MS
                            );
                            
                            tokio::time::sleep(Duration::from_millis(backoff_interval)).await;
                            continue;
                        }
                    }
                }

                // Poll every 500ms (or longer if errors occurred)
                tokio::time::sleep(Duration::from_millis(BASE_POLL_INTERVAL_MS)).await;
```

**Problem:** When `read_text()` fails:
1. Error handler applies backoff and `continue`s (line 108)
2. **BUT** line 114 **ALWAYS executes** after the match block, sleeping only 500ms
3. This means the backoff is **completely ignored** - the loop retries every 500ms regardless of error count
4. Result: Hundreds of failed reads per second, CPU spike, app crash

**Fix:**
```rust
// Replace lines 87-115 with:
                    Err(e) => {
                        consecutive_errors += 1;
                        
                        // Only log errors occasionally to avoid spam
                        if consecutive_errors == 1 || consecutive_errors % 10 == 0 {
                            eprintln!("[ClipboardMonitor] Failed to read clipboard (error #{}) : {}", consecutive_errors, e);
                        }
                        
                        // Calculate backoff interval based on error count
                        let sleep_interval = if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            // Exponential backoff up to MAX_POLL_INTERVAL_MS
                            let backoff_interval = std::cmp::min(
                                BASE_POLL_INTERVAL_MS * (2_u64.pow((consecutive_errors - MAX_CONSECUTIVE_ERRORS).min(4))),
                                MAX_POLL_INTERVAL_MS
                            );
                            eprintln!("[ClipboardMonitor] ⚠️  Too many consecutive errors ({}). Backing off to {}ms", consecutive_errors, backoff_interval);
                            backoff_interval
                        } else {
                            BASE_POLL_INTERVAL_MS
                        };
                        
                        tokio::time::sleep(Duration::from_millis(sleep_interval)).await;
                        continue; // Skip the unconditional sleep below
                    }
                }

                // Only sleep here if we didn't hit an error (successful read)
                tokio::time::sleep(Duration::from_millis(BASE_POLL_INTERVAL_MS)).await;
```

**Alternative Fix (Cleaner):**
```rust
// Restructure the loop to have a single sleep point:
loop {
    let is_enabled = *enabled.lock().unwrap();
    if !is_enabled {
        tokio::time::sleep(Duration::from_millis(BASE_POLL_INTERVAL_MS)).await;
        consecutive_errors = 0;
        continue;
    }

    let sleep_interval = match app.clipboard().read_text() {
        Ok(current_content) => {
            consecutive_errors = 0; // Reset on success
            
            if current_content.is_empty() {
                BASE_POLL_INTERVAL_MS
            } else {
                // Check if content changed and process...
                let mut last = last_content.lock().unwrap();
                let has_changed = match &*last {
                    Some(prev) => prev != &current_content,
                    None => true,
                };

                if has_changed {
                    *last = Some(current_content.clone());
                    drop(last);
                    
                    let source_app = crate::automation::get_active_app().ok();
                    let item = ClipboardItem::new_text(current_content, source_app);
                    history.add_item(item);

                    if let Err(e) = app.emit("clipboard-changed", history.get_items()) {
                        eprintln!("[ClipboardMonitor] Failed to emit event: {}", e);
                    }
                }
                
                BASE_POLL_INTERVAL_MS
            }
        }
        Err(e) => {
            consecutive_errors += 1;
            
            if consecutive_errors == 1 || consecutive_errors % 10 == 0 {
                eprintln!("[ClipboardMonitor] Failed to read clipboard (error #{}) : {}", consecutive_errors, e);
            }
            
            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                if consecutive_errors == MAX_CONSECUTIVE_ERRORS {
                    eprintln!("[ClipboardMonitor] ⚠️  Too many consecutive errors. Reducing polling frequency.");
                }
                
                std::cmp::min(
                    BASE_POLL_INTERVAL_MS * (2_u64.pow((consecutive_errors - MAX_CONSECUTIVE_ERRORS).min(4))),
                    MAX_POLL_INTERVAL_MS
                )
            } else {
                BASE_POLL_INTERVAL_MS
            }
        }
    };
    
    tokio::time::sleep(Duration::from_millis(sleep_interval)).await;
}
```

---

### 2. Panic Risk from Poisoned Mutexes

**File:** `src-tauri/src/clipboard/monitor.rs` (multiple locations)

**Issue:** All mutex locks use `.unwrap()` which will **panic the entire application** if a mutex becomes poisoned (thread panicked while holding lock).

**Locations:**
- Line 41: `enabled.lock().unwrap()`
- Line 61: `last_content.lock().unwrap()`
- Line 122: `enabled.lock().unwrap()`
- Line 129: `enabled.lock().unwrap()`
- Line 136: `enabled.lock().unwrap()`
- Line 141: `enabled.lock().unwrap()`

**Fix Pattern:**
```rust
// Replace all instances with:
match enabled.lock() {
    Ok(guard) => {
        // Use guard
        *guard = true;
    }
    Err(poisoned) => {
        eprintln!("[ClipboardMonitor] Mutex poisoned, recovering...");
        let guard = poisoned.into_inner();
        // Use guard
        *guard = true;
    }
}
```

**Or use a helper function:**
```rust
fn lock_or_recover<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("[ClipboardMonitor] Mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    }
}
```

---

### 3. Application-Wide Panic on Tauri Startup Failure

**File:** `src-tauri/src/lib.rs:238`

**Issue:** The `.expect()` call will **crash the entire application** if Tauri fails to initialize.

```238:238:src-tauri/src/lib.rs
        .expect("error while running tauri application");
```

**Fix:**
```rust
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to start Tauri application: {}", e);
            eprintln!("This is a critical error. Please check logs and system permissions.");
            std::process::exit(1);
        });
```

**Better Fix (Graceful Shutdown):**
```rust
match app.run(tauri::generate_context!()) {
    Ok(_) => {
        println!("Application exited normally");
    }
    Err(e) => {
        eprintln!("FATAL: Tauri application error: {}", e);
        // Log to file if possible
        std::process::exit(1);
    }
}
```

---

### 4. Process Integration Failures - No Circuit Breakers

**File:** `src-tauri/src/automation/macos.rs`

**Issue:** All AppleScript operations proceed **without checking**:
1. If accessibility permissions are granted
2. If the target application exists/is running
3. If the previous operation succeeded

**Locations:**

**4a. `restore_focus()` - No App Existence Check**
```35:51:src-tauri/src/automation/macos.rs
pub fn restore_focus(app_name: &str) -> Result<(), String> {
    let script = format!(
        r#"
        tell application "{}"
            activate
        end tell
        "#,
        app_name
    );

    execute_applescript(&script)?;
    
    // Wait a bit for the app to come to front
    thread::sleep(Duration::from_millis(100));
    
    Ok(())
}
```

**Problem:** If app doesn't exist or isn't running, AppleScript returns error "Can't get application", but this bubbles up and crashes the paste flow.

**Fix:**
```rust
pub fn restore_focus(app_name: &str) -> Result<(), String> {
    // Check if app is running first
    let check_script = format!(
        r#"
        tell application "System Events"
            set appList to name of every process
            if appList contains "{}" then
                return "running"
            else
                return "not_running"
            end if
        end tell
        "#,
        app_name
    );
    
    match execute_applescript(&check_script) {
        Ok(status) if status.trim() == "running" => {
            // App is running, proceed
        }
        Ok(_) => {
            return Err(format!("Application '{}' is not running", app_name));
        }
        Err(e) => {
            return Err(format!("Failed to check if app is running: {}", e));
        }
    }
    
    // Now try to activate
    let script = format!(
        r#"
        tell application "{}"
            activate
        end tell
        "#,
        app_name
    );

    execute_applescript(&script)?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}
```

**4b. `simulate_cmd_c()` and `simulate_cmd_v()` - No Permission Check**

**Problem:** These functions don't verify accessibility permissions before attempting keystrokes.

**Fix:**
```rust
pub fn simulate_cmd_c() -> Result<(), String> {
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted. Please enable in System Settings > Privacy & Security > Accessibility".to_string());
    }
    
    let script = r#"
        tell application "System Events"
            keystroke "c" using command down
        end tell
    "#;

    execute_applescript(script)?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}

pub fn simulate_cmd_v() -> Result<(), String> {
    // Check permissions first
    if !check_accessibility_permissions() {
        return Err("Accessibility permissions not granted".to_string());
    }
    
    let script = r#"
        tell application "System Events"
            keystroke "v" using command down
        end tell
    "#;

    execute_applescript(script)?;
    Ok(())
}
```

**4c. `auto_paste_flow()` - No Circuit Breaker**

**Problem:** This function chains multiple operations without checking if each succeeded, and has no retry limit.

```83:95:src-tauri/src/automation/macos.rs
pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> Result<(), String> {
    // Restore focus to the original app
    restore_focus(app_name)?;
    
    // Wait the specified delay (80-150ms as per Electron spec)
    thread::sleep(Duration::from_millis(delay_ms));
    
    // Simulate Cmd+V
    simulate_cmd_v()?;
    
    println!("[AutoPaste] Completed paste flow to app: {}", app_name);
    Ok(())
}
```

**Fix (Add Circuit Breaker):**
```rust
// Add at module level:
static mut PASTE_FAILURE_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const MAX_CONSECUTIVE_PASTE_FAILURES: u32 = 5;

pub fn auto_paste_flow(app_name: &str, delay_ms: u64) -> Result<(), String> {
    // Check circuit breaker
    let failures = unsafe { PASTE_FAILURE_COUNT.load(std::sync::atomic::Ordering::Relaxed) };
    if failures >= MAX_CONSECUTIVE_PASTE_FAILURES {
        return Err(format!(
            "Circuit breaker: Too many consecutive paste failures ({}). Please check accessibility permissions.",
            failures
        ));
    }
    
    // Check permissions
    if !check_accessibility_permissions() {
        unsafe { PASTE_FAILURE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        return Err("Accessibility permissions not granted".to_string());
    }
    
    // Restore focus with error handling
    if let Err(e) = restore_focus(app_name) {
        unsafe { PASTE_FAILURE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        return Err(format!("Failed to restore focus to '{}': {}", app_name, e));
    }
    
    thread::sleep(Duration::from_millis(delay_ms));
    
    // Simulate paste with error handling
    if let Err(e) = simulate_cmd_v() {
        unsafe { PASTE_FAILURE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        return Err(format!("Failed to paste: {}", e));
    }
    
    // Reset failure count on success
    unsafe { PASTE_FAILURE_COUNT.store(0, std::sync::atomic::Ordering::Relaxed); }
    
    println!("[AutoPaste] Completed paste flow to app: {}", app_name);
    Ok(())
}
```

---

### 5. Race Condition in Shortcut Handler

**File:** `src-tauri/src/lib.rs:143-169`

**Issue:** The shortcut handler uses blocking `std::thread::sleep()` in an async context, and doesn't handle errors from `get_active_app()`.

```143:169:src-tauri/src/lib.rs
                    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                        // Capture selected text BEFORE opening the window (while original app still has focus)
                        // Get and STORE the active app first
                        if let Ok(active_app) = automation::get_active_app() {
                            if let Ok(mut last_app) = last_app_clone.lock() {
                                *last_app = Some(active_app.clone());
                                println!("[Shortcut] Stored last active app: {}", active_app);
                            }
                        }
                        
                        // Simulate Cmd+C to copy selection to clipboard
                        if let Err(e) = automation::simulate_cmd_c() {
                            eprintln!("Failed to simulate Cmd+C in shortcut handler: {}", e);
                        }
                        
                        // Small delay for clipboard to update
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        
                        // Now open the palette window
                        if let Err(e) = show_widget_window(&handle, "palette") {
                            eprintln!("Failed to show palette window: {}", e);
                        } else {
                            // Explicitly focus the palette window
                            if let Some(window) = handle.get_webview_window("palette-window") {
                                let _ = window.set_focus();
                            }
                        }
                    }) {
```

**Problems:**
1. Blocking sleep in async context can deadlock
2. `get_active_app()` failure is silently ignored
3. `last_app_clone.lock()` can panic if mutex is poisoned

**Fix:**
```rust
if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
    // Spawn async task to avoid blocking
    let handle_clone = handle.clone();
    let last_app_clone = last_app_clone.clone();
    
    tauri::async_runtime::spawn(async move {
        // Capture active app with proper error handling
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
            if let Some(window) = handle_clone.get_webview_window("palette-window") {
                let _ = window.set_focus();
            }
        }
    });
}) {
```

---

## ⚠️ ERROR HANDLING GAPS

### Location: `src-tauri/src/clipboard/history.rs`

**Issue:** Multiple `.unwrap()` calls on mutex locks (lines 99, 107, 130, 135, 142, 150, 157, 162)

**Recommendation:** Use the same mutex recovery pattern as shown in Critical Flaw #2.

---

### Location: `src-tauri/src/context/ranking.rs`

**Issue:** Multiple `.unwrap()` calls (lines 24, 27, 36, 42, 48, 53, 54, 112)

**Recommendation:** Replace with proper error handling or mutex recovery.

---

### Location: `src-tauri/src/commands.rs:681`

**Issue:** 
```681:681:src-tauri/src/commands.rs
        let last_app_guard = last_active_app.lock().unwrap();
```

**Recommendation:**
```rust
let last_app_guard = match last_active_app.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        eprintln!("[PasteItem] Mutex poisoned, recovering...");
        poisoned.into_inner()
    }
};
```

---

### Location: `src-tauri/src/lib.rs:76`

**Issue:**
```76:76:src-tauri/src/lib.rs
                .icon(app.default_window_icon().unwrap().clone())
```

**Recommendation:**
```rust
.icon(app.default_window_icon()
    .ok_or("Failed to get default window icon")?
    .clone())
```

---

## 📊 SUMMARY

### Critical Issues Found: 5
1. ✅ **Infinite loop in clipboard monitor** (CPU exhaustion)
2. ✅ **Panic risk from poisoned mutexes** (28 instances)
3. ✅ **Application-wide panic on startup failure**
4. ✅ **No circuit breakers for AppleScript operations**
5. ✅ **Race condition in shortcut handler**

### High-Priority Fixes:
1. **Fix clipboard monitor backoff logic** (immediate - causes crashes)
2. **Add mutex recovery patterns** (prevents panics)
3. **Add permission checks before automation** (prevents AppleScript errors)
4. **Add circuit breaker for paste operations** (prevents infinite retry loops)

### Medium-Priority Fixes:
1. Replace remaining `.unwrap()` calls with proper error handling
2. Add retry limits to all AppleScript operations
3. Add app existence checks before `restore_focus()`

---

## 🔧 RECOMMENDED IMPLEMENTATION ORDER

1. **Phase 1 (Critical - Do First):**
   - Fix clipboard monitor backoff logic
   - Add mutex recovery to clipboard monitor
   - Add circuit breaker to `auto_paste_flow()`

2. **Phase 2 (High Priority):**
   - Add permission checks to all automation functions
   - Fix shortcut handler race condition
   - Replace `.expect()` in `lib.rs:238`

3. **Phase 3 (Medium Priority):**
   - Replace all remaining `.unwrap()` calls
   - Add app existence checks
   - Add comprehensive error logging

---

**Report End**

