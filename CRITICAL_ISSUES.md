# Critical Issues Analysis

## Issue #1: Runtime Panic - Cannot Start Runtime From Within Runtime
**Location**: `src-tauri/src/lib.rs:536, 544, 967`
**Severity**: CRITICAL - Causes app crash
**Root Cause**: 
- `handle.block_on()` called from within async context (line 536)
- `Runtime::new()` called when runtime already exists (lines 544, 967)
- Menu items and commands call sync wrapper which tries to block

**Fix Required**:
1. Use `tokio::task::spawn_blocking` for sync callers instead of `block_on`
2. Check if already in async context before blocking
3. Make menu handlers spawn async tasks

---

## Issue #2: Menu Items Blocking on Sync Function
**Location**: `src-tauri/src/lib.rs:92, 97, 118`
**Severity**: CRITICAL - Prevents menu items from working
**Root Cause**: 
- Menu event handlers call `show_widget_window()` (sync)
- Sync function tries to block on async code
- Causes panic when called from within runtime

**Fix Required**:
```rust
// Change from:
"palette" => {
    if let Err(e) = show_widget_window(app, "palette", false) {
        eprintln!("Failed to show palette: {}", e);
    }
}

// To:
"palette" => {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(window_lock) = app_handle.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
            if let Err(e) = show_widget_window_async(&app_handle, "palette", false, window_lock.inner().clone()).await {
                eprintln!("Failed to show palette: {}", e);
            }
        }
    });
}
```

---

## Issue #3: Command Handler Blocking
**Location**: `src-tauri/src/commands/window.rs:104`
**Severity**: CRITICAL - Prevents widget commands from working
**Root Cause**: 
- `show_widget` command is async but calls sync `show_widget_window()`
- Sync function blocks, causing deadlock/panic

**Fix Required**:
```rust
// Change from:
pub async fn show_widget(app: tauri::AppHandle, widget: String) -> CommandResult<()> {
    crate::show_widget_window(&app, &widget, false)
        .map_err(|e| format_window_error(&format!("show {} widget", widget), &e.to_string()))
}

// To:
pub async fn show_widget(app: tauri::AppHandle, widget: String) -> CommandResult<()> {
    if let Some(window_lock) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>() {
        crate::show_widget_window_async(&app, &widget, false, window_lock.inner().clone()).await
            .map_err(|e| format_window_error(&format!("show {} widget", widget), &e.to_string()))
    } else {
        Err("Window lock not available".to_string())
    }
}
```

---

## Issue #4: Window Cannot Become Key Despite Override
**Location**: `src-tauri/src/nswindow.rs` - `force_window_key()`
**Severity**: CRITICAL - Window focus not working
**Root Cause**: 
- Dynamic subclass override works (`canBecomeKeyWindow: true`)
- But `makeKeyAndOrderFront` still fails (`isKeyWindow: false` after call)
- Logs show: Line 885 - window visible, can become key, but not key after makeKey

**Possible Causes**:
1. Window delegate blocking key window
2. Content view not accepting first responder properly
3. Window level or collection behavior conflict
4. Timing issue - window not ready when makeKey is called

**Fix Required**:
1. Add delay after `show()` before calling `makeKeyWindow`
2. Verify content view accepts first responder
3. Check for window delegate that might block
4. Try `orderFront:` before `makeKeyWindow`

---

## Issue #5: Multiple Concurrent Shortcut Triggers
**Location**: `src-tauri/src/lib.rs:152-159` (shortcut registration)
**Severity**: HIGH - Causes race conditions
**Root Cause**: 
- Shortcut handler spawns async task
- Multiple shortcuts can trigger simultaneously
- Mutex lock works but doesn't prevent duplicate triggers

**Fix Required**:
1. Add debouncing to shortcut handler (ignore triggers within 100ms)
2. Use atomic flag to prevent concurrent execution
3. Ensure shortcut is only registered once

---

## Issue #6: Focus Stealing During Text Selection
**Location**: `src-tauri/src/automation/macos.rs` - `detect_text_selection()`
**Severity**: HIGH - Breaks user workflow
**Root Cause**: 
- `simulate_cmd_c()` or window operations activate app
- Focus moves from original app to our app
- Logs show: "FOCUS WAS STOLEN! Started with Some(\"Arc\"), ended with Some(\"productivity-widgets\")"

**Fix Required**:
1. Don't activate app during selection detection
2. Store original app and restore focus after detection
3. Use `NSWorkspace` to get active app without activating

---

## Issue #7: Sync Wrapper Implementation
**Location**: `src-tauri/src/lib.rs:530-555`
**Severity**: CRITICAL - Root cause of runtime panics
**Root Cause**: 
- Sync wrapper tries to use `block_on` or create new runtime
- Fails when called from async context
- Legacy fallback also has same issue

**Fix Required**:
```rust
// Replace sync wrapper with spawn_blocking approach:
fn show_widget_window(app: &tauri::AppHandle, widget: &str, has_selection: bool) -> Result<(), Box<dyn std::error::Error>> {
    // For sync callers, spawn blocking task
    let app_clone = app.clone();
    let widget_str = widget.to_string();
    let window_lock = app.try_state::<std::sync::Arc<tokio::sync::Mutex<()>>>()
        .ok_or("Window lock not available")?;
    let lock_clone = window_lock.inner().clone();
    
    // Use spawn_blocking to avoid runtime conflicts
    let rt = tokio::runtime::Handle::try_current()
        .ok_or("No tokio runtime available")?;
    
    rt.spawn(async move {
        if let Err(e) = show_widget_window_async(&app_clone, &widget_str, has_selection, lock_clone).await {
            eprintln!("Failed to show window: {}", e);
        }
    });
    
    // Return immediately (fire and forget for sync callers)
    Ok(())
}
```

---

## Priority Order for Fixes:
1. **Issue #7** - Fix sync wrapper (blocks all other windows)
2. **Issue #2** - Fix menu handlers (blocks menu access)
3. **Issue #3** - Fix command handler (blocks widget commands)
4. **Issue #4** - Fix window focus (core functionality)
5. **Issue #6** - Fix focus stealing (UX issue)
6. **Issue #5** - Fix concurrent triggers (stability)

---

## Testing Checklist:
- [ ] Menu items can open windows without panic
- [ ] Command palette shortcut works consistently
- [ ] Widget commands (`show_widget`) work
- [ ] Window becomes key and receives focus
- [ ] Focus is not stolen from original app
- [ ] No runtime panics in logs
- [ ] Multiple rapid shortcut presses don't cause issues

