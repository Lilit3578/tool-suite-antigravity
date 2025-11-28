# UX Polish Checklist
## Frontend Quality Assurance Audit

---

## 🔴 CRITICAL: Visual Glitches & Window Management

### Issue 1: Background Window Flash & Transparency Race Condition

**Component:** `App.tsx`, `CommandPalette.tsx`, Window Initialization

**Problem:**
- Window is created with `transparent(true)` in Rust, but React may render before transparency is fully applied
- CSS `background: transparent !important` may not prevent initial white flash
- The wrapper div in `App.tsx` uses inline styles that may override Tailwind classes
- `pointerEvents: 'none'` on CommandPalette wrapper may cause interaction issues

**CSS/Config Fix:**

```css
/* Add to index.css - Prevent flash of unstyled content */
html, body, #root {
  background: transparent !important;
  background-color: transparent !important;
  /* Ensure transparency is applied immediately */
  opacity: 0;
  transition: opacity 0.1s ease-in;
}

html.loaded, body.loaded, #root.loaded {
  opacity: 1;
}
```

**Code Fix - App.tsx:**
```typescript
// Add to App.tsx useEffect
useEffect(() => {
  // Prevent flash by ensuring transparency is ready
  if (currentWidget === "palette") {
    // Add loaded class after a microtask to ensure DOM is ready
    requestAnimationFrame(() => {
      document.documentElement.classList.add("loaded");
      document.body.classList.add("loaded");
      const root = document.getElementById("root");
      if (root) root.classList.add("loaded");
    });
  }
}, [currentWidget]);
```

**Code Fix - CommandPalette.tsx:**
```typescript
// Remove pointerEvents: 'none' from wrapper div (line 295)
// Change from:
<div style={{
    width: '550px',
    height: '328px',
    background: 'transparent',
    position: 'relative',
    pointerEvents: 'none'  // ❌ REMOVE THIS
}}>

// To:
<div style={{
    width: '550px',
    height: '328px',
    background: 'transparent',
    position: 'relative'
}}>
```

**Rust Fix - lib.rs:**
```rust
// Ensure window is hidden until transparency is ready
// In show_widget_window function, after creating window:
if widget == "palette" {
    window.set_visible(false)?;
    // Small delay to ensure transparency is applied
    std::thread::sleep(std::time::Duration::from_millis(10));
    window.set_visible(true)?;
}
```

---

### Issue 2: Tailwind Styles Not Applying

**Component:** `CommandPalette.tsx`, `command.tsx`

**Problem:**
- Inline styles override Tailwind classes
- Custom hover states in `CommandItem` conflict with `data-[selected=true]` states
- Background colors use CSS variables that may not be defined at render time

**CSS/Config Fix:**

```css
/* Add to index.css - Ensure CSS variables are available immediately */
:root {
  /* ... existing variables ... */
  /* Add explicit fallbacks */
  --ink-200: rgba(20, 20, 20, 0.04);
  --ink-1000: rgba(20, 20, 20, 1);
}

/* Fix CommandItem hover/selection conflict */
[cmdk-item][data-selected="true"] {
  background-color: hsl(var(--accent)) !important;
  color: hsl(var(--accent-foreground)) !important;
}

[cmdk-item]:not([data-selected="true"]):hover {
  background-color: var(--ink-200) !important;
  color: var(--ink-1000) !important;
}
```

**Code Fix - command.tsx:**
```typescript
// Remove inline style backgroundColor/color from CommandItem
// The data-selected attribute should handle all styling
const CommandItem = React.forwardRef<...>(({ className, ...props }, ref) => {
  // ❌ REMOVE isHovered state and inline styles
  // ✅ Let cmdk handle selection state via data-selected
  return (
    <CommandPrimitive.Item
      ref={ref}
      className={cn(
        "relative flex cursor-default gap-3 select-none items-center rounded-lg px-4 py-1 font-light outline-none",
        "data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        "data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground",
        "hover:bg-ink-200 hover:text-ink-1000", // Add hover via Tailwind
        "[&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:text-ink-800",
        className
      )}
      {...props}
    />
  );
});
```

---

## 🔴 CRITICAL: Input Handling & Navigation

### Issue 3: Keyboard Navigation Not Working

**Component:** `CommandPalette.tsx`, `command.tsx`

**Problem:**
- Window may not be focused when CommandPalette mounts
- `autoFocus` on CommandInput only works if window already has focus
- Blur handler may interfere with keyboard events
- Focus may be lost to hidden window frame

**Logic Fix:**

1. **Ensure window focus before React mount:**
   - Window should be focused in Rust BEFORE showing
   - Add explicit focus call after window is visible

2. **Fix focus management in CommandPalette:**
   - Use `useEffect` to focus input after window is confirmed focused
   - Add keyboard event listeners at window level as fallback

**Code Fix - CommandPalette.tsx:**

```typescript
// Add after line 28 (state declarations)
const inputRef = useRef<HTMLInputElement>(null);

// Add new useEffect after line 59 (capture text effect)
useEffect(() => {
  // Ensure window is focused and input receives focus
  const ensureFocus = async () => {
    try {
      const window = getCurrentWindow();
      await window.setFocus();
      
      // Wait for next frame to ensure DOM is ready
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          inputRef.current?.focus();
        });
      });
    } catch (e) {
      console.error("Failed to focus window:", e);
    }
  };

  ensureFocus();

  // Also focus on window focus event
  const handleWindowFocus = () => {
    requestAnimationFrame(() => {
      inputRef.current?.focus();
    });
  };

  window.addEventListener("focus", handleWindowFocus);
  return () => window.removeEventListener("focus", handleWindowFocus);
}, []);

// Update CommandInput to use ref
<CommandInput
  ref={inputRef}
  placeholder="search..."
  value={query}
  onValueChange={setQuery}
  autoFocus
/>
```

**Code Fix - lib.rs (window creation):**

```rust
// After window.show() and before returning
if widget == "palette" {
    window.set_focus()?;
    // Small delay to ensure focus is set
    std::thread::sleep(std::time::Duration::from_millis(50));
    window.set_focus()?; // Double-focus to ensure it sticks
}
```

**Code Fix - Blur Handler (CommandPalette.tsx line 100):**

```typescript
const handleBlur = (e: FocusEvent) => {
  // Don't hide if focus is moving to a child element
  const relatedTarget = e.relatedTarget as HTMLElement;
  if (relatedTarget && document.body.contains(relatedTarget)) {
    // Focus is moving within the app, don't hide
    console.log("[Blur] Focus moving within app, ignoring");
    return;
  }

  // Check if click-through was just enabled (rapid mouse movement)
  const timeSinceLastIgnore = Date.now() - lastIgnoreTime;

  if (lastIgnoreState && timeSinceLastIgnore < CLICK_THROUGH_DEBOUNCE) {
    console.log("[Blur] Ignored - rapid mouse movement");
    return;
  }

  // Schedule hide after delay
  console.log("[Blur] Scheduling hide in", BLUR_HIDE_DELAY, "ms");
  blurTimeout = setTimeout(() => {
    console.log("[Blur] Hiding palette");
    api.hidePaletteWindow().catch(e => console.error("Failed to hide palette:", e));
  }, BLUR_HIDE_DELAY);
};
```

---

### Issue 4: Scrolling Broken in CommandList

**Component:** `command.tsx` (CommandList)

**Problem:**
- `CommandList` has `overflow-y-auto` but may lose scroll capability if focus is lost
- Keyboard navigation (arrow keys) may not scroll list into view
- `cmdk` library may not handle scrolling for off-screen items

**Code Fix - command.tsx:**

```typescript
const CommandList = React.forwardRef<
  React.ElementRef<typeof CommandPrimitive.List>,
  React.ComponentPropsWithoutRef<typeof CommandPrimitive.List>
>(({ className, ...props }, ref) => {
  const listRef = React.useRef<HTMLDivElement>(null);

  // Auto-scroll selected item into view
  React.useEffect(() => {
    const list = listRef.current;
    if (!list) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Let cmdk handle arrow keys, but ensure scrolling
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        requestAnimationFrame(() => {
          const selected = list.querySelector('[data-selected="true"]');
          if (selected) {
            selected.scrollIntoView({ block: "nearest", behavior: "smooth" });
          }
        });
      }
    };

    list.addEventListener("keydown", handleKeyDown);
    return () => list.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <CommandPrimitive.List
      ref={(node) => {
        listRef.current = node;
        if (typeof ref === "function") ref(node);
        else if (ref) ref.current = node;
      }}
      className={cn(
        "max-h-[300px] overflow-y-auto overflow-x-hidden",
        "scrollbar-thin scrollbar-thumb-ink-400 scrollbar-track-transparent", // Add custom scrollbar
        className
      )}
      {...props}
    />
  );
});
```

**CSS Fix - Add smooth scrolling:**

```css
/* Add to index.css */
[cmdk-list] {
  scroll-behavior: smooth;
  /* Ensure scrollbar is visible on macOS */
  -webkit-overflow-scrolling: touch;
}

/* Custom scrollbar styling for macOS */
[cmdk-list]::-webkit-scrollbar {
  width: 8px;
}

[cmdk-list]::-webkit-scrollbar-track {
  background: transparent;
}

[cmdk-list]::-webkit-scrollbar-thumb {
  background: var(--ink-400);
  border-radius: 4px;
}

[cmdk-list]::-webkit-scrollbar-thumb:hover {
  background: var(--ink-500);
}
```

---

## 🟡 HIGH PRIORITY: Feedback & Latency

### Issue 5: UI Blocks During Async Operations

**Component:** `CommandPalette.tsx` (handleExecuteAction, handleOpenWidget)

**Problem:**
- `handleExecuteAction` is async and blocks UI while waiting for Rust
- No loading state shown to user
- No optimistic UI updates
- Translation widget has loading state, but actions don't

**Code Fix - Add Loading States:**

```typescript
// Add to state declarations (line 30)
const [isExecuting, setIsExecuting] = useState(false);
const [executingActionId, setExecutingActionId] = useState<string | null>(null);

// Update handleExecuteAction (line 184)
async function handleExecuteAction(actionId: string, actionType: string) {
  setSelectedActionId(actionId);
  setIsError(false);
  setIsExecuting(true);
  setExecutingActionId(actionId);

  try {
    // ... existing capture text logic ...

    // Show optimistic loading state
    setPopoverContent("Processing...");
    setPopoverOpen(true);

    const result = await api.executeAction({
      action_type: actionType as any,
      params: {
        text: textToUse,
      },
    });

    setPopoverContent(result.result);
    setIsExecuting(false);
    setExecutingActionId(null);

    // Record usage (don't await - fire and forget)
    api.recordCommandUsage(actionId).catch(e => console.error("Failed to record usage:", e));

    setTimeout(() => {
      setPopoverOpen(false);
      setSelectedActionId(null);
    }, 3000);
  } catch (e) {
    console.error("Action execution failed:", e);
    setPopoverContent(`Error: ${e}`);
    setIsError(true);
    setIsExecuting(false);
    setExecutingActionId(null);
    // ... rest of error handling ...
  }
}

// Update PopoverContent to show loading state
<PopoverContent
  side="right"
  align="center"
  className={`w-auto max-w-[250px] ${isError ? 'border-red-500 bg-red-50' : ''} ${isExecuting ? 'opacity-75' : ''}`}
  style={{ pointerEvents: 'auto' }}
>
  <div className={`body text-sm ${isError ? 'text-red-600' : ''}`}>
    {isExecuting && executingActionId === cmd.id ? (
      <div className="flex items-center gap-2">
        <div className="animate-spin h-4 w-4 border-2 border-ink-400 border-t-transparent rounded-full" />
        <span>Processing...</span>
      </div>
    ) : (
      popoverContent
    )}
  </div>
</PopoverContent>
```

**Code Fix - Optimistic Widget Opening:**

```typescript
// Update handleOpenWidget (line 165)
async function handleOpenWidget(widgetId: string) {
  try {
    // Hide palette immediately (optimistic)
    await getCurrentWindow().hide();

    // Fire widget opening and usage recording in parallel (don't await)
    Promise.all([
      (async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('show_widget', { widget: widgetId });
      })(),
      api.recordCommandUsage(`widget_${widgetId}`)
    ]).catch(e => {
      console.error("Failed to open widget or record usage:", e);
    });
  } catch (e) {
    console.error("Failed to open widget:", widgetId, e);
  }
}
```

---

### Issue 6: Command Loading Has No Feedback

**Component:** `CommandPalette.tsx` (loadCommands effect)

**Problem:**
- Commands load asynchronously but UI shows empty state immediately
- No skeleton loader or loading indicator

**Code Fix - Add Loading State:**

```typescript
// Add to state (line 25)
const [isLoadingCommands, setIsLoadingCommands] = useState(true);

// Update loadCommands effect (line 62)
useEffect(() => {
  const loadCommands = async () => {
    setIsLoadingCommands(true);
    try {
      const items = await api.getCommandItems(capturedText || undefined);
      setCommands(items);
    } catch (e) {
      console.error("Failed to load commands:", e);
      setCommands([]); // Ensure empty state on error
    } finally {
      setIsLoadingCommands(false);
    }
  };

  loadCommands();
}, [capturedText]);

// Update CommandEmpty to show loading
<CommandEmpty>
  {isLoadingCommands ? (
    <div className="flex flex-col items-center gap-2 py-4">
      <div className="animate-spin h-6 w-6 border-2 border-ink-400 border-t-transparent rounded-full" />
      <span className="text-ink-700">Loading commands...</span>
    </div>
  ) : (
    <Button variant="link">no commands found</Button>
  )}
</CommandEmpty>
```

---

## 🟢 POLISH RECOMMENDATIONS

### Delight Feature 1: Smooth Animations

**Add micro-interactions for native feel:**

```css
/* Add to index.css */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

[cmdk-item] {
  animation: fadeIn 0.15s ease-out;
}

[cmdk-item][data-selected="true"] {
  transform: scale(1.02);
  transition: transform 0.1s ease-out, background-color 0.1s ease-out;
}

/* Popover entrance animation */
[data-radix-popover-content] {
  animation: fadeIn 0.2s ease-out;
}
```

### Delight Feature 2: Focus Trapping

**Ensure keyboard navigation stays within palette:**

```typescript
// Add to CommandPalette.tsx
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    // Trap Escape key to close
    if (e.key === "Escape") {
      api.hidePaletteWindow().catch(e => console.error("Failed to hide:", e));
    }
  };

  window.addEventListener("keydown", handleKeyDown);
  return () => window.removeEventListener("keydown", handleKeyDown);
}, []);
```

### Delight Feature 3: Haptic-like Feedback

**Add subtle visual feedback for interactions:**

```css
/* Add to index.css */
[cmdk-item]:active {
  transform: scale(0.98);
  transition: transform 0.05s ease-out;
}

/* Ripple effect on selection */
@keyframes ripple {
  0% {
    transform: scale(0);
    opacity: 0.5;
  }
  100% {
    transform: scale(2);
    opacity: 0;
  }
}

[cmdk-item][data-selected="true"]::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background: currentColor;
  opacity: 0.1;
  animation: ripple 0.3s ease-out;
}
```

### Delight Feature 4: Search Highlighting

**Highlight matching text in command labels:**

```typescript
// Add helper function to CommandPalette.tsx
const highlightMatch = (text: string, query: string) => {
  if (!query.trim()) return text;
  
  const parts = text.split(new RegExp(`(${query})`, 'gi'));
  return parts.map((part, i) => 
    part.toLowerCase() === query.toLowerCase() ? (
      <mark key={i} className="bg-accent/20 text-accent-foreground">{part}</mark>
    ) : part
  );
};

// Use in CommandItem rendering
<span>{highlightMatch(cmd.label, query)}</span>
```

### Delight Feature 5: Keyboard Shortcuts Display

**Show keyboard shortcuts in command items:**

```typescript
// Add to CommandItem rendering
<CommandItemUI ...>
  {getIcon(cmd)}
  <span>{cmd.label}</span>
  {cmd.shortcut && (
    <CommandShortcut>{cmd.shortcut}</CommandShortcut>
  )}
</CommandItemUI>
```

---

## 📋 Implementation Priority

1. **P0 (Critical - Blocks Core Functionality):**
   - Issue 1: Background window flash
   - Issue 3: Keyboard navigation not working
   - Issue 4: Scrolling broken

2. **P1 (High - Degrades UX Significantly):**
   - Issue 2: Tailwind styles not applying
   - Issue 5: UI blocks during async operations
   - Issue 6: Command loading has no feedback

3. **P2 (Nice to Have - Polish):**
   - All Delight Features (1-5)

---

## 🧪 Testing Checklist

After implementing fixes, verify:

- [ ] Window opens without white flash
- [ ] Keyboard arrow keys navigate through commands
- [ ] Enter key executes selected command
- [ ] Escape key closes palette
- [ ] Scrolling works when list overflows
- [ ] Focus remains in input when typing
- [ ] Loading states appear during async operations
- [ ] Popover appears with smooth animation
- [ ] No visual glitches when switching between commands
- [ ] Window closes on blur (after delay)
- [ ] Click-through works correctly outside palette area

---

## 📝 Notes

- All fixes maintain existing functionality
- No breaking changes to API contracts
- All changes follow existing code patterns
- TypeScript types are preserved
- Accessibility (a11y) is maintained/improved

