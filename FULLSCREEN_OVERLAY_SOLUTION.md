# Fullscreen Overlay Window Management - Technical Solution

## Root Cause Analysis

### Why macOS Hides Windows by Default

macOS uses **Spaces** (virtual desktops) and **Mission Control** to manage windows across different contexts. When an application enters fullscreen mode, macOS creates a dedicated Space for that application. By default, windows from other applications are **excluded** from this fullscreen Space due to:

1. **NSWindowCollectionBehavior**: Controls how windows interact with Spaces
   - Default behavior: Windows stay on their original Space
   - Fullscreen apps create isolated Spaces
   - Windows without proper flags are invisible in fullscreen Spaces

2. **NSWindowLevel**: Determines window stacking order
   - Normal windows (level 0) are below fullscreen apps
   - Status bar level (25) appears above fullscreen apps
   - Menu bar level (24) also works but is reserved for system UI

3. **Space Isolation**: Fullscreen apps create isolated Spaces that other windows cannot enter without explicit permission

### Current Implementation Issues

After analyzing the codebase, I identified **three critical problems**:

1. **Incomplete Widget Configuration**: Only the "palette" widget receives fullscreen overlay configuration. Other widgets (translator, currency, clipboard, settings) are NOT configured, causing them to appear on the desktop Space instead of the active fullscreen Space.

2. **Configuration Timing**: The native configuration is applied AFTER `window.show()` in some code paths, which can cause macOS to reject the configuration or apply it incorrectly.

3. **Missing Widget Inheritance**: When widgets are opened from the Command Palette, they don't inherit the fullscreen overlay behavior.

---

## Step-by-Step Implementation

### Step 1: Create Unified Configuration Function

**File**: `src-tauri/src/nswindow.rs`

The current `ensure_fullscreen_overlay_config()` function is correct but needs to be applied to ALL widgets, not just the palette.

### Step 2: Apply Configuration to All Widgets

**File**: `src-tauri/src/lib.rs`

Modify `show_widget_window_async()` and `show_widget_window_create_new_async()` to apply fullscreen overlay configuration to ALL widgets, not just the palette.

### Step 3: Ensure Configuration Happens Before Showing

**Critical**: The configuration MUST happen while the window is hidden, BEFORE `window.show()` is called.

---

## Code Implementation

### Fix 1: Update `show_widget_window_async()` - Apply to All Widgets

```rust:src-tauri/src/lib.rs
// In show_widget_window_async(), around line 420-470
if widget == "palette" {
    // ... existing palette-specific code ...
} else {
    // CRITICAL FIX: Apply fullscreen overlay to ALL widgets, not just palette
    #[cfg(target_os = "macos")]
    {
        match nswindow::ensure_fullscreen_overlay_config(&window) {
            Ok(_) => println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' configured for fullscreen overlay", widget),
            Err(e) => {
                eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Fullscreen overlay configuration failed for '{}': {}", widget, e);
            }
        }
    }
    
    // Also use Tauri's built-in always_on_top for consistency
    window.set_always_on_top(true)
        .map_err(|e| format!("Failed to set always on top: {}", e))?;
    
    println!("🔵 [DEBUG] [show_widget_window] Showing widget '{}'...", widget);
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    
    // Small delay for window to become visible
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Give the window focus so user can type immediately
    if let Err(e) = window.set_focus() {
        eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Failed to give window focus: {}", e);
    } else {
        println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' has focus - ready for typing", widget);
    }
}
```

### Fix 2: Update `show_widget_window_create_new_async()` - Configure Before Showing

```rust:src-tauri/src/lib.rs
// In show_widget_window_create_new_async(), around line 745-792
// CRITICAL: Configure ALL widgets for fullscreen overlay, not just palette
if widget == "palette" || widget == "translator" || widget == "currency" || widget == "clipboard" || widget == "settings" {
    println!("🔵 [DEBUG] [show_widget_window] Configuring widget '{}' for fullscreen overlay...", widget);
    
    // CRITICAL: Configure BEFORE showing the window
    #[cfg(target_os = "macos")]
    {
        match nswindow::ensure_fullscreen_overlay_config(&window) {
            Ok(_) => println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' fullscreen overlay configuration complete", widget),
            Err(e) => {
                eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Fullscreen overlay configuration failed for '{}': {}", widget, e);
                // Continue anyway - Tauri's always_on_top might still work
            }
        }
    }
    
    // Also use Tauri's built-in always_on_top as a fallback
    window.set_always_on_top(true)
        .map_err(|e| format!("Failed to set always on top: {}", e))?;
    
    // Use Tauri's normal window.show() - it may briefly activate the app, but that's acceptable
    println!("🔵 [DEBUG] [show_widget_window] Showing widget '{}'...", widget);
    window.show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    
    // Small delay for window to appear
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Give the window focus so user can type immediately
    if let Err(e) = window.set_focus() {
        eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Failed to give window focus: {}", e);
    } else {
        println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' has focus - ready for typing", widget);
    }
    
    // Verify visibility using Tauri API
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        println!("🔵 [DEBUG] [show_widget_window] ✅ Widget '{}' is visible!", widget);
    } else {
        eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Widget '{}' is NOT visible!", widget);
        // Try showing again
        window.show().ok();
    }
    
    println!("🔵 [DEBUG] [show_widget_window] Widget '{}' creation complete", widget);
}
```

### Fix 3: Update Legacy Function for Consistency

```rust:src-tauri/src/lib.rs
// In show_widget_window_legacy(), around line 583-588
} else {
    println!("🔵 [DEBUG] [show_widget_window] Showing non-palette widget '{}'...", widget);
    
    // CRITICAL FIX: Apply fullscreen overlay to all widgets
    #[cfg(target_os = "macos")]
    {
        match nswindow::ensure_fullscreen_overlay_config(&window) {
            Ok(_) => println!("🔵 [DEBUG] [show_widget_window] ✓ Widget '{}' configured for fullscreen overlay", widget),
            Err(e) => {
                eprintln!("🔴 [DEBUG] [show_widget_window] ⚠️  Fullscreen overlay configuration failed for '{}': {}", widget, e);
            }
        }
    }
    
    window.set_always_on_top(true)
        .map_err(|e| format!("Failed to set always on top: {}", e))?;
    
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    
    // Give window focus
    window.set_focus().map_err(|e| format!("Failed to set focus: {}", e))?;
}
```

### Fix 4: Helper Function for Widget List (Optional but Recommended)

Add this helper function to make the code more maintainable:

```rust:src-tauri/src/lib.rs
// Add near the top of the file, after imports
/// Check if a widget should have fullscreen overlay behavior
fn widget_needs_fullscreen_overlay(widget: &str) -> bool {
    matches!(widget, "palette" | "translator" | "currency" | "clipboard" | "settings")
}
```

Then use it in the code:
```rust
if widget_needs_fullscreen_overlay(widget) {
    // Apply fullscreen overlay configuration
}
```

---

## Thinking Process (Chain of Thought)

### 1. macOS Spaces and Full Screen Architecture

**How macOS Handles Spaces:**
- Each Space is a separate virtual desktop
- Fullscreen apps create a dedicated Space
- Windows belong to specific Spaces based on `NSWindowCollectionBehavior`
- Windows can appear on multiple Spaces with `CanJoinAllSpaces` (0x1)
- Windows can overlay fullscreen apps with `FullScreenAuxiliary` (0x80)

**Why Windows Disappear:**
- Default behavior: Windows stay on their original Space
- Fullscreen apps create isolated Spaces
- Without proper flags, windows are invisible in fullscreen Spaces

### 2. Tauri's `set_always_on_top(true)` Limitation

**Analysis:**
- `set_always_on_top(true)` only affects window stacking within the SAME Space
- It does NOT make windows appear in fullscreen Spaces
- It does NOT set the required `NSWindowCollectionBehavior` flags
- It does NOT set the window level to appear above fullscreen apps

**Conclusion:** Tauri's `set_always_on_top()` is **insufficient** for fullscreen overlay. Native macOS configuration is **required**.

### 3. Correct NSWindowLevel Selection

**Options:**
- `NSNormalWindowLevel` (0): Below fullscreen apps ❌
- `NSFloatingWindowLevel` (3): Still below fullscreen apps ❌
- `NSSubmenuWindowLevel` (3): Below fullscreen apps ❌
- `NSTornOffMenuWindowLevel` (3): Below fullscreen apps ❌
- `NSMainMenuWindowLevel` (24): Reserved for menu bar ❌
- `NSStatusWindowLevel` (25): **Perfect for overlays** ✅
- `NSModalPanelWindowLevel` (8): Below fullscreen apps ❌
- `NSPopUpMenuWindowLevel` (101): Too high, reserved for popups ❌

**Conclusion:** `NSStatusWindowLevel` (25) is the correct choice for utility overlays that need to appear over fullscreen apps.

### 4. NSWindowCollectionBehavior Flags

**Required Flags:**
- `CanJoinAllSpaces` (0x1): Window appears on all Spaces, including fullscreen
- `FullScreenAuxiliary` (0x80): Window can overlay fullscreen apps
- **Combined**: 0x81 (0x1 | 0x80)

**Flags to Avoid:**
- `MoveToActiveSpace` (0x2): Conflicts with `CanJoinAllSpaces` and can cause crashes
- `Stationary` (0x4): Prevents window from moving between Spaces (opposite of what we want)

**Conclusion:** Use `CanJoinAllSpaces | FullScreenAuxiliary` (0x81) for all overlay widgets.

### 5. Lightweight Rust Implementation

**Current Implementation Analysis:**
- ✅ Uses `cocoa` and `objc` crates (standard for macOS FFI)
- ✅ Properly dispatches to main thread (required for AppKit)
- ✅ Minimal configuration (only level and collection behavior)
- ✅ Safe error handling

**Optimization Opportunities:**
- The current implementation is already lightweight
- No need for additional dependencies
- Configuration is applied only when needed
- Thread-safe with proper synchronization

**Conclusion:** The current native implementation in `nswindow.rs` is optimal. The fix is to **apply it to all widgets**, not just the palette.

---

## Verification Checklist

After implementing the fixes, verify:

- [ ] Command Palette appears over fullscreen apps
- [ ] Translator widget appears over fullscreen apps
- [ ] Currency Converter appears over fullscreen apps
- [ ] Clipboard History appears over fullscreen apps
- [ ] Settings window appears over fullscreen apps
- [ ] All widgets get focus immediately (no click required)
- [ ] Windows appear on the active Space (not desktop)
- [ ] No crashes or errors in console logs

---

## Summary

**Root Cause:** Only the palette widget was configured for fullscreen overlay. Other widgets lacked the required `NSWindowLevel` (25) and `NSWindowCollectionBehavior` (0x81) configuration.

**Solution:** Apply the existing `ensure_fullscreen_overlay_config()` function to ALL widgets (palette, translator, currency, clipboard, settings) in all code paths (async, sync, legacy).

**Key Principle:** All utility/overlay windows that need to appear over fullscreen apps must have:
1. Window level 25 (NSStatusWindowLevel)
2. Collection behavior 0x81 (CanJoinAllSpaces | FullScreenAuxiliary)
3. Configuration applied BEFORE showing the window

