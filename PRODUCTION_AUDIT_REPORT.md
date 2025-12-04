# Production Codebase Audit Report

**Date:** 2024-12-03  
**Auditor:** Staff Software Engineer  
**Focus:** Critical paths, production-readiness, silent killers

---

## 1. Critical Issues & Fixes (Must Fix)

### Issue 1: Panic Risk - `.unwrap()` in Regex Compilation

**File:** `src-tauri/src/core/features/unit_converter.rs` (Lines 335, 345, 355)

**Risk:** If regex patterns are malformed (unlikely but possible), the app will crash with a panic. In production, this kills the entire app.

**The Fix:**

```rust
// BEFORE (Lines 335, 345, 355)
let re1 = Regex::new(r"^([+-]?\d+(?:\.\d+)?)\s*([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)$").unwrap();
let re2 = Regex::new(r"^([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)\s*([+-]?\d+(?:\.\d+)?)$").unwrap();
let re3 = Regex::new(r"([+-]?\d+(?:\.\d+)?)").unwrap();

// AFTER
// Compile regex patterns once at module level (lazy_static or const)
use once_cell::sync::Lazy;

static RE_PATTERN_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-]?\d+(?:\.\d+)?)\s*([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)$")
        .expect("Failed to compile regex pattern 1") // Safe: compile-time constant
});

static RE_PATTERN_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)\s*([+-]?\d+(?:\.\d+)?)$")
        .expect("Failed to compile regex pattern 2")
});

static RE_PATTERN_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([+-]?\d+(?:\.\d+)?)")
        .expect("Failed to compile regex pattern 3")
});

// Then use:
if let Some(caps) = RE_PATTERN_1.captures(&normalized_text) {
    // ...
}
```

**Alternative (if lazy_static not available):** Use `Regex::new().map_err(|e| format!("Invalid regex: {}", e))?` and propagate error.

---

### Issue 2: Panic Risk - `.unwrap()` in Window Operations

**File:** `src-tauri/src/system/window/nswindow.rs` (Lines 141, 178)

**Risk:** If mutex is poisoned or window handle is invalid, app crashes.

**The Fix:**

```rust
// BEFORE (Line 141)
if let Some(f) = closure.lock().unwrap().take() {

// AFTER
if let Some(f) = closure.lock()
    .map_err(|e| {
        eprintln!("[Window] Mutex poisoned: {}", e);
        e.into_inner()
    })
    .and_then(|mut guard| Ok(guard.take()))
    .unwrap_or(None) {
    // ...
}

// BEFORE (Line 178)
let ns_window = window_clone.ns_window().unwrap() as id;

// AFTER
let ns_window = window_clone.ns_window()
    .ok_or_else(|| "Window handle not available".to_string())? as id;
```

---

### Issue 3: Regex DoS Vulnerability - No Input Length Limits

**File:** `src-tauri/src/core/context/detection.rs` (Line 20)  
**File:** `src-tauri/src/core/context/category.rs` (Line 38)

**Risk:** If user copies 10MB of text, regex operations will freeze the app for seconds or crash it. This is a classic DoS vector.

**The Fix:**

```rust
// BEFORE (detection.rs, line 20)
pub fn detect_currency(text: &str) -> Option<CurrencyInfo> {
    // ... regex operations on potentially huge text
}

// AFTER
pub fn detect_currency(text: &str) -> Option<CurrencyInfo> {
    // Truncate input to prevent DoS attacks
    const MAX_INPUT_LENGTH: usize = 1000;
    let truncated_text = if text.len() > MAX_INPUT_LENGTH {
        &text[..MAX_INPUT_LENGTH]
    } else {
        text
    };
    
    // ... rest of function using truncated_text
}

// BEFORE (category.rs, line 38)
pub fn detect_content_category(text: &str) -> Option<ContextCategory> {
    let text_lower = text.to_lowercase(); // Creates full copy of potentially huge string
    // ... regex operations
}

// AFTER
pub fn detect_content_category(text: &str) -> Option<ContextCategory> {
    // Truncate and lowercase only what we need
    const MAX_INPUT_LENGTH: usize = 1000;
    let truncated = if text.len() > MAX_INPUT_LENGTH {
        &text[..MAX_INPUT_LENGTH]
    } else {
        text
    };
    let text_lower = truncated.to_lowercase();
    
    // ... rest of function
}
```

---

### Issue 4: Memory Leak - Unclean setTimeout in CommandPalette

**File:** `src/components/CommandPalette.tsx` (Lines 412, 437, 449)

**Risk:** If component unmounts while setTimeout is pending, the timeout continues to run, causing memory leaks and potential state updates on unmounted components.

**The Fix:**

```typescript
// BEFORE (Lines 412, 437, 449)
setTimeout(() => {
    setPopoverOpen(false);
    setSelectedActionId(null);
}, 3000);

// AFTER
const timeoutId = setTimeout(() => {
    setPopoverOpen(false);
    setSelectedActionId(null);
}, 3000);

// Store timeout IDs in a ref and clear on unmount
const timeoutRefs = useRef<Set<NodeJS.Timeout>>(new Set());

// In the function:
const timeoutId = setTimeout(() => {
    setPopoverOpen(false);
    setSelectedActionId(null);
}, 3000);
timeoutRefs.current.add(timeoutId);

// Add cleanup effect:
useEffect(() => {
    return () => {
        timeoutRefs.current.forEach(id => clearTimeout(id));
        timeoutRefs.current.clear();
    };
}, []);
```

---

### Issue 5: Memory Leak - Unclean setTimeout in ClipboardHistoryWidget

**File:** `src/components/widgets/ClipboardHistoryWidget.tsx` (Lines 47, 76)

**Risk:** Same as above - timeouts not cleaned up on unmount.

**The Fix:**

```typescript
// BEFORE (Line 47)
useEffect(() => {
    const timer = setTimeout(() => {
        // ... focus logic
    }, 100);
    // Missing cleanup!
}, []);

// AFTER
useEffect(() => {
    const timer = setTimeout(() => {
        if (commandRef.current) {
            commandRef.current.focus();
        }
        const firstFocusable = document.querySelector('input, button, [tabindex="0"]') as HTMLElement;
        if (firstFocusable) {
            firstFocusable.focus();
        }
    }, 100);
    
    return () => clearTimeout(timer); // ✅ Cleanup
}, []);

// BEFORE (Line 76)
setTimeout(() => {
    isPastingRef.current = false;
}, 500);

// AFTER
const pasteTimeoutRef = useRef<NodeJS.Timeout | null>(null);

// In handleSelect:
if (pasteTimeoutRef.current) {
    clearTimeout(pasteTimeoutRef.current);
}
pasteTimeoutRef.current = setTimeout(() => {
    isPastingRef.current = false;
    pasteTimeoutRef.current = null;
}, 500);

// Add cleanup:
useEffect(() => {
    return () => {
        if (pasteTimeoutRef.current) {
            clearTimeout(pasteTimeoutRef.current);
        }
    };
}, []);
```

---

## 2. Logical Improvements (Should Fix)

### Issue 6: Negative Number Validation Missing

**File:** `src-tauri/src/core/features/unit_converter.rs` (Line 83)

**Risk:** Converting "-5km" to miles returns a negative result, which is physically meaningless for length/mass/volume. User gets confusing output.

**The Fix:**

```rust
// BEFORE (Line 83)
let (amount, source_unit) = parse_unit_from_text(text)?;

// AFTER
let (amount, source_unit) = parse_unit_from_text(text)?;

// Validate negative numbers for physical quantities
match action_type {
    ActionType::ConvertToMM | ActionType::ConvertToCM | ActionType::ConvertToM 
    | ActionType::ConvertToKM | ActionType::ConvertToIN | ActionType::ConvertToFT 
    | ActionType::ConvertToYD | ActionType::ConvertToMI => {
        // Length cannot be negative
        if amount < 0.0 {
            return Err("Length cannot be negative. Please provide a positive value.".to_string());
        }
    }
    ActionType::ConvertToMG | ActionType::ConvertToG | ActionType::ConvertToKG 
    | ActionType::ConvertToOZ | ActionType::ConvertToLB => {
        // Mass cannot be negative
        if amount < 0.0 {
            return Err("Mass cannot be negative. Please provide a positive value.".to_string());
        }
    }
    ActionType::ConvertToML | ActionType::ConvertToL | ActionType::ConvertToFlOz 
    | ActionType::ConvertToCup | ActionType::ConvertToPint | ActionType::ConvertToQuart 
    | ActionType::ConvertToGal => {
        // Volume cannot be negative
        if amount < 0.0 {
            return Err("Volume cannot be negative. Please provide a positive value.".to_string());
        }
    }
    // Temperature and Speed can be negative (valid use cases)
    _ => {}
}
```

---

### Issue 7: Floating Point Precision Edge Case

**File:** `src-tauri/src/core/features/unit_converter.rs` (Line 621-673)

**Risk:** The `format_number` function has a potential issue with very large numbers when converting to `i64`. Numbers beyond `i64::MAX` will panic.

**The Fix:**

```rust
// BEFORE (Line 634)
let integer_part = rounded.trunc() as i64;

// AFTER
// Handle overflow for very large numbers
let integer_part = if rounded.abs() > i64::MAX as f64 {
    // For numbers beyond i64::MAX, use string formatting directly
    return format!("{:.2}", rounded)
        .replace(".00", "")
        .trim_end_matches('0')
        .trim_end_matches('.');
} else {
    rounded.trunc() as i64
};
```

**Note:** This is edge case handling. For widget use cases, numbers this large are unlikely, but defensive programming prevents crashes.

---

## 3. Performance Safeguards

### Issue 8: Regex Performance on Large Inputs (Already Addressed in Issue 3)

**Status:** Fixed by adding input truncation in Issue 3.

**Additional Optimization:** Consider caching compiled regex patterns (see Issue 1).

---

### Issue 9: useLayoutEffect Dependency Array

**File:** `src/components/CommandPalette.tsx` (Line 208)

**Status:** ✅ **CORRECT** - Dependency array `[query]` is correct. Only fires when query changes, not on every render.

**Verification:**
```typescript
useLayoutEffect(() => {
    if (query === '' && commandListRef.current) {
        commandListRef.current.scrollTo({ top: 0, behavior: 'instant' });
    }
}, [query]); // ✅ Correct - only depends on query
```

---

### Issue 10: Hardcoded Error Strings

**File:** Multiple files

**Risk:** Error messages are hardcoded in English. For internationalization, these should be in a constants file.

**The Fix:**

**Create:** `src-tauri/src/shared/errors.rs`

```rust
pub mod errors {
    pub const ERR_MISSING_TEXT_PARAM: &str = "Missing 'text' parameter";
    pub const ERR_NEGATIVE_LENGTH: &str = "Length cannot be negative. Please provide a positive value.";
    pub const ERR_NEGATIVE_MASS: &str = "Mass cannot be negative. Please provide a positive value.";
    pub const ERR_NEGATIVE_VOLUME: &str = "Volume cannot be negative. Please provide a positive value.";
    pub const ERR_UNSUPPORTED_ACTION: &str = "Unsupported action type";
    pub const ERR_CANNOT_PARSE_UNIT: &str = "Could not parse unit from text";
    pub const ERR_WINDOW_HANDLE_UNAVAILABLE: &str = "Window handle not available";
}
```

**Then use:**
```rust
use crate::shared::errors::*;

// Instead of:
.ok_or("Missing 'text' parameter")?

// Use:
.ok_or(ERR_MISSING_TEXT_PARAM)?
```

---

## Summary

### Critical (Must Fix Before Production):
1. ✅ Replace `.unwrap()` with proper error handling (3 instances in unit_converter.rs, 2 in nswindow.rs)
2. ✅ Add input length limits to prevent Regex DoS (detection.rs, category.rs)
3. ✅ Fix memory leaks from uncleaned timeouts (CommandPalette.tsx, ClipboardHistoryWidget.tsx)

### Important (Should Fix):
4. ✅ Add negative number validation for physical quantities
5. ✅ Handle floating point overflow edge cases
6. ✅ Extract hardcoded error strings to constants file

### Verified Correct:
- ✅ useLayoutEffect dependency array is correct
- ✅ Event listeners are properly cleaned up in most places
- ✅ Floating point rounding logic is sound

---

## Testing Checklist

After applying fixes, test:

- [ ] Copy 10MB of text → app should not freeze (truncation works)
- [ ] Convert "-5km" → should show friendly error, not negative result
- [ ] Rapidly open/close CommandPalette → no memory leaks (check DevTools Memory tab)
- [ ] Convert very large numbers (1e20) → should not panic
- [ ] Regex compilation failures → should log error, not panic

---

**Priority:** Fix Critical issues (1-3) immediately. Important issues (4-6) can be addressed in next patch.

