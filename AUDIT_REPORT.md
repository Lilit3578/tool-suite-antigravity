# 🔍 Complete Forensic Audit Report: NSNonactivatingPanel Implementation

## Executive Summary

Your current implementation has **multiple critical flaws** that prevent the floating panel from appearing over fullscreen windows. The code attempts to convert an `NSWindow` to an `NSPanel` using runtime class mutation, creates dynamic Objective-C subclasses (causing stack overflow), uses private APIs, and tries to make a non-activating panel become key (contradictory behavior).

---

## ❌ CRITICAL PROBLEMS FOUND

### 1. **Runtime Class Mutation (object_setClass)**
**Location:** `src/nswindow.rs:165-168, 240-243`

**Problem:** Using `object_setClass()` to convert an `NSWindow` to `NSPanel` at runtime. This is fragile and can cause undefined behavior.

```rust
object_setClass(ns_window, panel_class);
```

**Why it fails:**
- Runtime class mutation breaks Cocoa's internal state
- The window's internal structure doesn't match the new class
- Can cause crashes or silent failures

**Fix:** Create a REAL `NSPanel` from the start, not convert an `NSWindow`.

---

### 2. **Dynamic Subclass Creation (Stack Overflow Source)**
**Location:** `src/nswindow.rs:183-236`

**Problem:** Creates a unique Objective-C subclass for EACH window instance using `objc_allocateClassPair()`. This can cause:
- Stack overflow (creating too many classes)
- Memory leaks (classes are never deallocated)
- Recursion issues

```rust
let subclass_name = format!("NonActivatingPanel_{:p}", ns_window);
let new_class = objc_allocateClassPair(current_class, subclass_name_cstr.as_ptr(), 0);
```

**Why it fails:**
- Each window gets its own class, leading to class explosion
- Classes are registered globally and never cleaned up
- Can cause recursion if called multiple times

**Fix:** Use a single static subclass or override methods differently.

---

### 3. **Private API Usage (CGSSetWindowLevel)**
**Location:** `src/nswindow.rs:303-315`

**Problem:** Uses private Core Graphics API `CGSSetWindowLevel()` which:
- May be rejected by App Store
- Can break in future macOS versions
- Not guaranteed to work

```rust
extern "C" {
    fn CGSSetWindowLevel(connection: u32, window_id: u32, level: i32) -> i32;
    fn CGSMainConnectionID() -> u32;
}
```

**Fix:** Use only public `NSWindow.setLevel()` API.

---

### 4. **Trying to Make Non-Activating Panel Key**
**Location:** `src/nswindow.rs:409-432`, `src/lib.rs:316-370`, `src/lib.rs:588-609`

**Problem:** Multiple calls to `force_window_key()`, `set_focus()`, and `makeKeyAndOrderFront()` which contradict the non-activating panel behavior.

**Files affected:**
- `src/nswindow.rs:444-463` - `force_window_key()` function
- `src/lib.rs:316` - Calls `force_window_key()`
- `src/lib.rs:366` - Fallback to `window.set_focus()`
- `src/lib.rs:588` - Calls `force_window_key()` again
- `src/commands/window.rs:96` - `focus_palette_window()` calls `set_focus()`
- `src/commands/palette.rs:82-112` - Multiple `set_focus()` calls

**Why it fails:**
- Non-activating panels should NEVER become key
- Making it key activates the app (breaks fullscreen overlay)
- Contradicts the `setBecomesKeyOnlyIfNeeded: YES` setting

**Fix:** Remove ALL calls to `set_focus()`, `force_window_key()`, and `makeKey`. Use only `orderFrontRegardless`.

---

### 5. **Wrong Collection Behavior Flags**
**Location:** `src/nswindow.rs:328-331`

**Problem:** Uses contradictory flags: `Stationary` + `IgnoresCycle` + `FullScreenAuxiliary`. The correct combination for fullscreen overlays is ONLY:
- `CanJoinAllSpaces` (0x1)
- `FullScreenAuxiliary` (0x80)

```rust
let desired_bits: u64 = NSWindowCollectionBehaviorCanJoinAllSpaces 
    | NSWindowCollectionBehaviorFullScreenAuxiliary  // 0x80 - CRITICAL for fullscreen
    | NSWindowCollectionBehaviorStationary          // ❌ REMOVE
    | NSWindowCollectionBehaviorIgnoresCycle;      // ❌ REMOVE
```

**Why it fails:**
- `Stationary` prevents the window from moving between spaces
- `IgnoresCycle` can conflict with fullscreen behavior
- Only `CanJoinAllSpaces | FullScreenAuxiliary` is needed

**Fix:** Use only `CanJoinAllSpaces | FullScreenAuxiliary`.

---

### 6. **Post-Show Window Manipulation**
**Location:** `src/lib.rs:580-609`

**Problem:** Manipulates window level and properties AFTER showing the window. This breaks ordering on fullscreen.

```rust
// STEP 4: Ensure window level is maximum and force window to front
if let Err(e) = nswindow::ensure_maximum_window_level(&window) {
    // ...
}
```

**Why it fails:**
- Window level must be set BEFORE showing
- Changing level after show() can cause flickering or invisibility
- Fullscreen apps may reject the change

**Fix:** Configure ALL properties BEFORE calling `show()`.

---

### 7. **Creating Panel After WebView Attachment**
**Location:** `src/lib.rs:1103-1189` (create_floating_panel function)

**Problem:** Tries to extract a webview from an existing Tauri window and attach it to a new panel. This won't work correctly because:
- The webview is already attached to an NSWindow
- You can't move a view between windows easily
- The original window still exists

**Fix:** Create the NSPanel FIRST, then attach the webview during window creation.

---

### 8. **Multiple Redundant Window Operations**
**Location:** Throughout `src/lib.rs`

**Problem:** Multiple redundant calls to:
- `orderFrontRegardless()` (called 3+ times)
- `orderFront()` (called after `orderFrontRegardless`)
- Window state checks (excessive logging)
- Focus attempts (shouldn't be called at all)

**Fix:** Simplify to a single `orderFrontRegardless()` call after configuration.

---

## 📋 DETAILED FILE-BY-FILE BREAKDOWN

### `src/nswindow.rs`
**Lines to REMOVE:**
- 165-168: `object_setClass()` call (runtime class mutation)
- 183-236: Dynamic subclass creation (stack overflow source)
- 240-243: Second `object_setClass()` call
- 303-318: Private API `CGSSetWindowLevel()` usage
- 328-331: Wrong collection behavior flags (remove Stationary and IgnoresCycle)
- 409-432: `make_window_key_main_thread()` function (shouldn't make key)
- 444-463: `force_window_key()` function (remove entirely)

**Lines to MODIFY:**
- 136-402: `convert_to_panel_and_configure_main_thread()` - Remove class mutation, use real NSPanel
- 474-497: `ensure_maximum_window_level()` - Should only be called BEFORE show()

---

### `src/lib.rs`
**Lines to REMOVE:**
- 316-370: All `force_window_key()` calls and focus attempts
- 366: Fallback `window.set_focus()` call
- 580-609: Post-show window manipulation
- 588: `force_window_key()` call after show()
- 1063: Fallback `set_focus()` call
- 1103-1189: `create_floating_panel()` function (wrong approach)

**Lines to MODIFY:**
- 492-617: `show_widget_window_async()` - Remove all focus/activation calls
- 792-1084: `show_widget_window_create_new_async()` - Configure BEFORE show, remove focus calls

---

### `src/commands/window.rs`
**Lines to REMOVE:**
- 92-99: `focus_palette_window()` function (shouldn't focus non-activating panel)

---

### `src/commands/palette.rs`
**Lines to REMOVE:**
- 82-93: Multiple `set_focus()` calls after capture
- 110-112: Additional `set_focus()` calls

---

## 🏗️ CORRECTED ARCHITECTURE PLAN

### 1. **Window Creation Flow**
```
1. Create Tauri window (HIDDEN)
   - Use WebviewWindowBuilder with visible(false)
   - Set transparent=true, decorations=false
   
2. Get NSWindow pointer from Tauri window
   - Use window.ns_window()
   
3. Create REAL NSPanel (not convert NSWindow)
   - Allocate new NSPanel with same frame
   - Transfer webview content view to new panel
   - Release old NSWindow
   
4. Configure NSPanel (BEFORE showing)
   - setBecomesKeyOnlyIfNeeded: YES
   - setFloatingPanel: YES
   - setLevel: NSStatusWindowLevel (25)
   - setCollectionBehavior: CanJoinAllSpaces | FullScreenAuxiliary
   - setOpaque: NO
   - setAlphaValue: 1.0
   - setHasShadow: YES
   
5. Show panel
   - orderFrontRegardless() (NOT makeKey)
   - Verify visibility
```

### 2. **Allowed APIs**
✅ `NSPanel.alloc().initWithContentRect:styleMask:backing:defer:`
✅ `setBecomesKeyOnlyIfNeeded:`
✅ `setFloatingPanel:`
✅ `setLevel:`
✅ `setCollectionBehavior:`
✅ `orderFrontRegardless`
✅ `setOpaque:`
✅ `setAlphaValue:`
✅ `setHasShadow:`

### 3. **Disallowed APIs**
❌ `object_setClass()` - No runtime class mutation
❌ `objc_allocateClassPair()` - No dynamic subclasses
❌ `CGSSetWindowLevel()` - No private APIs
❌ `makeKey()` / `makeKeyAndOrderFront()` - No activation
❌ `set_focus()` - No Tauri focus calls
❌ `force_window_key()` - No key window attempts
❌ `setCollectionBehavior:` with Stationary or IgnoresCycle

---

## 🔧 PATCH LIST

### File: `src/nswindow.rs`
**Changes:**
1. Remove `object_setClass()` usage
2. Remove dynamic subclass creation
3. Remove private API `CGSSetWindowLevel()`
4. Fix collection behavior (only CanJoinAllSpaces | FullScreenAuxiliary)
5. Remove `force_window_key()` function
6. Simplify to only configure, don't convert

### File: `src/lib.rs`
**Changes:**
1. Remove ALL `force_window_key()` calls
2. Remove ALL `set_focus()` calls
3. Remove post-show window manipulation
4. Configure window BEFORE showing
5. Use only `orderFrontRegardless()` after show
6. Remove `create_floating_panel()` function

### File: `src/commands/window.rs`
**Changes:**
1. Remove `focus_palette_window()` function

### File: `src/commands/palette.rs`
**Changes:**
1. Remove all `set_focus()` calls

---

## 🎯 ROOT CAUSE ANALYSIS

The fundamental issue is that you're trying to:
1. **Convert** an NSWindow to NSPanel (should CREATE NSPanel)
2. **Make** a non-activating panel key (contradictory)
3. **Manipulate** window after showing (breaks ordering)
4. **Use** private APIs and runtime hacks (fragile)

The correct approach is:
1. **Create** a real NSPanel from the start
2. **Configure** it as non-activating BEFORE showing
3. **Never** make it key or activate the app
4. **Use** only public, supported APIs

---

## ✅ VERIFICATION CHECKLIST

After fixes, verify:
- [ ] Panel appears over fullscreen apps
- [ ] App does NOT activate when panel shows
- [ ] Panel does NOT become key window
- [ ] Panel accepts text input (canBecomeKeyWindow returns YES for text fields)
- [ ] No stack overflow errors
- [ ] No private API usage
- [ ] Only uses CanJoinAllSpaces | FullScreenAuxiliary
- [ ] All configuration happens BEFORE show()
- [ ] Only uses orderFrontRegardless() to show

---

## 📝 NOTES

- The `panel.rs` file has a good structure but isn't being used correctly
- Tauri's window creation API doesn't directly support NSPanel, so we need to create it manually
- The webview must be transferred from NSWindow to NSPanel carefully
- Non-activating panels can still accept text input if `canBecomeKeyWindow` returns YES (which it should for text fields)

