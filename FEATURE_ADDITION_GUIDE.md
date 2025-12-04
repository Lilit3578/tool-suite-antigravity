# Feature Addition Guide

This guide provides step-by-step instructions for adding a new feature to the Productivity Widgets application.

## Table of Contents

1. [Required Information](#required-information)
2. [Feature Types](#feature-types)
3. [Implementation Checklist](#implementation-checklist)
4. [Step-by-Step Implementation](#step-by-step-implementation)
5. [Code Examples](#code-examples)
6. [Testing Checklist](#testing-checklist)

---

## Required Information

Before implementing a new feature, you must provide the following information:

### 1. Feature Specification

- **Feature ID**: Unique identifier (e.g., `"calculator"`, `"notes"`, `"weather"`)
- **Feature Name**: Human-readable name (e.g., `"Calculator"`, `"Quick Notes"`, `"Weather Widget"`)
- **Feature Description**: Brief description of what the feature does

### 2. Command Items

Define what commands should appear in the Command Palette:

#### Widget Command (if feature has a dedicated window)
- **Command ID**: `"widget_{feature_id}"` (e.g., `"widget_calculator"`)
- **Label**: Display name (e.g., `"Calculator"`)
- **Description**: Optional tooltip text

#### Action Commands (if feature has quick actions)
- List of action commands with:
  - **Command ID**: Unique identifier (e.g., `"calculate_add"`)
  - **Label**: Display name (e.g., `"Add Numbers"`)
  - **Description**: Optional description
  - **Action Type**: Enum variant (must be added to `ActionType`)

### 3. Window Configuration (if applicable)

If the feature requires a dedicated window:

- **Window Dimensions**: Width × Height (e.g., `700 × 550`)
- **Window Title**: Display title
- **Transparent**: `true` or `false`
- **Decorations**: `true` or `false` (window chrome)
- **Resizable**: `true` or `false`

### 4. Backend Commands

List all Tauri commands the feature needs:

- **Command Name**: Function name (e.g., `calculate_expression`)
- **Parameters**: Input parameters with types
- **Return Type**: Expected return type
- **Description**: What the command does

### 5. Frontend Component Requirements

- **UI Components Needed**: List of shadcn/ui components (e.g., `Card`, `Input`, `Button`)
- **State Management**: What state needs to be managed (local vs global)
- **API Calls**: Which backend commands to call

### 6. Data Models

- **Input Types**: Structures for command parameters
- **Output Types**: Structures for command responses
- **Settings**: Any feature-specific settings that should be persisted

### 7. Dependencies

- **Rust Crates**: External dependencies needed (e.g., `reqwest` for HTTP, `serde` for serialization)
- **Frontend Packages**: npm packages needed (if any)

---

## Feature Types

Features can be categorized into three types:

### Type 1: Widget-Only Feature
- Has a dedicated window/widget
- No quick actions in Command Palette
- Example: Settings Widget

### Type 2: Action-Only Feature
- No dedicated window
- Provides quick actions executed directly
- Example: System commands (lock screen, sleep)

### Type 3: Hybrid Feature
- Has both a widget window AND quick actions
- Example: Translator (widget + quick translate actions)

---

## Implementation Checklist

Use this checklist to ensure all steps are completed:

### Backend (Rust)
- [ ] Create feature module file: `src-tauri/src/core/features/{feature_id}.rs`
- [ ] Implement `Feature` trait
- [ ] Add command handler functions
- [ ] Register commands in `lib.rs`
- [ ] Add window dimensions in `lib.rs` (if widget)
- [ ] Add `ActionType` variants (if actions)
- [ ] Update `shared/types.rs` with data structures
- [ ] Add dependencies to `Cargo.toml` (if needed)

### Frontend (React/TypeScript)
- [ ] Create widget component: `src/components/widgets/{FeatureName}Widget.tsx`
- [ ] Add API function: `src/logic/api/tauri.ts`
- [ ] Add TypeScript types: `src/logic/types/index.ts`
- [ ] Update `App.tsx` routing (if widget)
- [ ] Add to window dimensions mapping (if widget)

### Integration
- [ ] Register feature in `core/features.rs`
- [ ] Test Command Palette integration
- [ ] Test widget window (if applicable)
- [ ] Test quick actions (if applicable)

---

## Step-by-Step Implementation

### Step 1: Define Data Types

**File**: `src-tauri/src/shared/types.rs`

Add your feature's data structures:

```rust
// Example: Calculator feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateResponse {
    pub result: f64,
    pub error: Option<String>,
}
```

**File**: `src/logic/types/index.ts`

Add corresponding TypeScript types:

```typescript
// Example: Calculator feature
export interface CalculateRequest {
  expression: string;
}

export interface CalculateResponse {
  result: number;
  error?: string;
}
```

### Step 2: Add Action Types (if applicable)

**File**: `src-tauri/src/shared/types.rs`

Add variants to the `ActionType` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionType {
    // ... existing variants ...
    CalculateAdd,
    CalculateSubtract,
    CalculateMultiply,
    CalculateDivide,
}
```

### Step 3: Create Backend Feature Module

**File**: `src-tauri/src/core/features/{feature_id}.rs`

```rust
//! {Feature Name} feature
//!
//! {Brief description of what the feature does}

use crate::shared::types::*;
use crate::shared::settings::AppSettings;
use crate::core::context;
use super::Feature;
use std::collections::HashMap;

pub struct {FeatureName}Feature;

impl Feature for {FeatureName}Feature {
    fn id(&self) -> &str {
        "{feature_id}"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        // If feature has a widget window
        vec![CommandItem {
            id: "widget_{feature_id}".to_string(),
            label: "{Feature Name}".to_string(),
            description: Some("Open {feature name} widget".to_string()),
            action_type: None,
            widget_type: Some("{feature_id}".to_string()),
        }]
        // If no widget, return empty vec
        // vec![]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        // If feature has quick actions
        vec![
            CommandItem {
                id: "{action_id_1}".to_string(),
                label: "{Action Label 1}".to_string(),
                description: Some("{Action description}".to_string()),
                action_type: Some(ActionType::{ActionTypeVariant1}),
                widget_type: None,
            },
            // Add more actions as needed
        ]
        // If no actions, return empty vec
        // vec![]
    }
    
    fn execute_action(
        &self,
        action_type: &ActionType,
        context: &context::Context,
        settings: &AppSettings,
    ) -> Result<ActionResult, String> {
        match action_type {
            ActionType::{ActionTypeVariant1} => {
                // Implementation for action 1
                Ok(ActionResult::Success {
                    message: Some("Action completed".to_string()),
                })
            }
            _ => Err(format!("Unknown action type: {:?}", action_type)),
        }
    }
    
    fn rank_commands(
        &self,
        commands: &mut [CommandItem],
        context: &context::Context,
        settings: &AppSettings,
    ) {
        // Optional: Implement custom ranking logic
        // Default: No ranking (commands appear in order defined)
    }
}

// Command handler functions
#[tauri::command]
pub async fn {command_name}(
    request: {RequestType},
    app: tauri::AppHandle,
) -> Result<{ResponseType}, String> {
    // Implementation
    Ok({ResponseType} {
        // ... fields
    })
}
```

### Step 4: Register Feature

**File**: `src-tauri/src/core/features.rs`

Add your feature to the `get_all_features()` function:

```rust
pub fn get_all_features() -> Vec<Box<dyn Feature>> {
    vec![
        Box::new(translator::TranslatorFeature),
        Box::new(currency::CurrencyFeature),
        Box::new(clipboard::ClipboardFeature),
        Box::new({feature_id}::{FeatureName}Feature), // Add here
    ]
}
```

### Step 5: Register Commands

**File**: `src-tauri/src/lib.rs`

Add your command handlers to the `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    core::features::{feature_id}::{command_name},
])
```

### Step 6: Add Window Configuration (if widget)

**File**: `src-tauri/src/lib.rs`

Add window dimensions in `show_widget_window_async` and `show_widget_window_create_new_async`:

```rust
let (width, height, _title, _transparent, _decorations) = match widget {
    "palette" => (550, 328, "Command Palette", true, false),
    "clipboard" => (500, 400, "Clipboard History", false, false),
    "translator" => (700, 550, "Translator", false, false),
    "currency" => (500, 400, "Currency Converter", false, false),
    "settings" => (800, 600, "Settings", false, false),
    "{feature_id}" => ({width}, {height}, "{Window Title}", {transparent}, {decorations}),
    _ => (600, 400, "Widget", false, false),
};
```

### Step 7: Create Frontend Widget Component (if widget)

**File**: `src/components/widgets/{FeatureName}Widget.tsx`

```typescript
import { useEffect, useState } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
// Import other UI components as needed

export function {FeatureName}Widget() {
  const [state, setState] = useState<{StateType}>({/* initial state */});

  useEffect(() => {
    // Load initial data if needed
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const result = await api.{commandName}({/* params */});
      setState(result);
    } catch (error) {
      console.error("Failed to load data:", error);
    }
  };

  const handleAction = async () => {
    try {
      const result = await api.{commandName}({/* params */});
      // Handle result
    } catch (error) {
      console.error("Action failed:", error);
    }
  };

  return (
    <Card className="p-6">
      <h2 className="text-2xl font-bold mb-4">{Feature Name}</h2>
      {/* UI implementation */}
    </Card>
  );
}
```

### Step 8: Add Frontend API Functions

**File**: `src/logic/api/tauri.ts`

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { {RequestType}, {ResponseType} } from "../types";

export const api = {
  // ... existing functions ...
  
  async {commandName}(request: {RequestType}): Promise<{ResponseType}> {
    return await invoke("{command_name}", { request });
  },
};
```

### Step 9: Update App Routing (if widget)

**File**: `src/App.tsx`

Add routing for your widget:

```typescript
// In the widget routing section
if (widget === "{feature_id}") {
  return <{FeatureName}Widget />;
}
```

### Step 10: Add Dependencies (if needed)

**File**: `src-tauri/Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...
{package_name} = "{version}"
```

**File**: `package.json` (if frontend dependency needed)

```json
{
  "dependencies": {
    "{package-name}": "{version}"
  }
}
```

---

## Code Examples

### Example 1: Simple Widget-Only Feature (Calculator)

**Backend**: `src-tauri/src/core/features/calculator.rs`

```rust
//! Calculator feature
//!
//! Provides basic calculator functionality.

use crate::shared::types::*;
use crate::shared::settings::AppSettings;
use crate::core::context;
use super::Feature;

pub struct CalculatorFeature;

impl Feature for CalculatorFeature {
    fn id(&self) -> &str {
        "calculator"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_calculator".to_string(),
            label: "Calculator".to_string(),
            description: Some("Open calculator widget".to_string()),
            action_type: None,
            widget_type: Some("calculator".to_string()),
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        vec![] // No quick actions
    }
    
    fn execute_action(
        &self,
        _action_type: &ActionType,
        _context: &context::Context,
        _settings: &AppSettings,
    ) -> Result<ActionResult, String> {
        Err("Calculator has no quick actions".to_string())
    }
    
    fn rank_commands(
        &self,
        _commands: &mut [CommandItem],
        _context: &context::Context,
        _settings: &AppSettings,
    ) {
        // No custom ranking
    }
}

#[tauri::command]
pub async fn calculate_expression(
    expression: String,
) -> Result<f64, String> {
    // Simple evaluation (use a proper math parser in production)
    // This is just an example
    match expression.parse::<f64>() {
        Ok(num) => Ok(num),
        Err(_) => Err("Invalid expression".to_string()),
    }
}
```

**Frontend**: `src/components/widgets/CalculatorWidget.tsx`

```typescript
import { useState } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Input } from "../ui/input";
import { Button } from "../ui/button";

export function CalculatorWidget() {
  const [expression, setExpression] = useState("");
  const [result, setResult] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleCalculate = async () => {
    try {
      setError(null);
      const calculated = await api.calculateExpression(expression);
      setResult(calculated);
    } catch (err) {
      setError(err as string);
      setResult(null);
    }
  };

  return (
    <Card className="p-6">
      <h2 className="text-2xl font-bold mb-4">Calculator</h2>
      <Input
        value={expression}
        onChange={(e) => setExpression(e.target.value)}
        placeholder="Enter expression"
        className="mb-4"
      />
      <Button onClick={handleCalculate}>Calculate</Button>
      {result !== null && <p className="mt-4 text-2xl">Result: {result}</p>}
      {error && <p className="mt-4 text-red-500">Error: {error}</p>}
    </Card>
  );
}
```

### Example 2: Action-Only Feature (System Commands)

**Backend**: `src-tauri/src/core/features/system.rs`

```rust
//! System commands feature
//!
//! Provides quick system actions.

use crate::shared::types::*;
use crate::shared::settings::AppSettings;
use crate::core::context;
use super::Feature;

pub struct SystemFeature;

impl Feature for SystemFeature {
    fn id(&self) -> &str {
        "system"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![] // No widget
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        vec![
            CommandItem {
                id: "system_lock".to_string(),
                label: "Lock Screen".to_string(),
                description: Some("Lock your Mac".to_string()),
                action_type: Some(ActionType::LockScreen),
                widget_type: None,
            },
            CommandItem {
                id: "system_sleep".to_string(),
                label: "Sleep".to_string(),
                description: Some("Put Mac to sleep".to_string()),
                action_type: Some(ActionType::Sleep),
                widget_type: None,
            },
        ]
    }
    
    fn execute_action(
        &self,
        action_type: &ActionType,
        _context: &context::Context,
        _settings: &AppSettings,
    ) -> Result<ActionResult, String> {
        match action_type {
            ActionType::LockScreen => {
                // Implementation to lock screen
                std::process::Command::new("pmset")
                    .args(&["displaysleepnow"])
                    .output()
                    .map_err(|e| format!("Failed to lock screen: {}", e))?;
                Ok(ActionResult::Success {
                    message: Some("Screen locked".to_string()),
                })
            }
            ActionType::Sleep => {
                // Implementation to sleep
                std::process::Command::new("pmset")
                    .args(&["sleepnow"])
                    .output()
                    .map_err(|e| format!("Failed to sleep: {}", e))?;
                Ok(ActionResult::Success {
                    message: Some("Mac sleeping".to_string()),
                })
            }
            _ => Err(format!("Unknown action type: {:?}", action_type)),
        }
    }
    
    fn rank_commands(
        &self,
        _commands: &mut [CommandItem],
        _context: &context::Context,
        _settings: &AppSettings,
    ) {
        // No custom ranking
    }
}
```

### Example 3: Hybrid Feature (Notes)

**Backend**: `src-tauri/src/core/features/notes.rs`

```rust
//! Notes feature
//!
//! Provides quick note-taking with widget and quick actions.

use crate::shared::types::*;
use crate::shared::settings::AppSettings;
use crate::core::context;
use super::Feature;

pub struct NotesFeature;

impl Feature for NotesFeature {
    fn id(&self) -> &str {
        "notes"
    }
    
    fn widget_commands(&self) -> Vec<CommandItem> {
        vec![CommandItem {
            id: "widget_notes".to_string(),
            label: "Quick Notes".to_string(),
            description: Some("Open notes widget".to_string()),
            action_type: None,
            widget_type: Some("notes".to_string()),
        }]
    }
    
    fn action_commands(&self) -> Vec<CommandItem> {
        vec![
            CommandItem {
                id: "note_new".to_string(),
                label: "New Note".to_string(),
                description: Some("Create a new note".to_string()),
                action_type: Some(ActionType::NewNote),
                widget_type: None,
            },
        ]
    }
    
    fn execute_action(
        &self,
        action_type: &ActionType,
        context: &context::Context,
        _settings: &AppSettings,
    ) -> Result<ActionResult, String> {
        match action_type {
            ActionType::NewNote => {
                // Use selected text if available
                let content = context.selected_text.clone().unwrap_or_default();
                // Create note with content
                Ok(ActionResult::OpenWidget {
                    widget: "notes".to_string(),
                    data: Some(serde_json::json!({ "content": content })),
                })
            }
            _ => Err(format!("Unknown action type: {:?}", action_type)),
        }
    }
    
    fn rank_commands(
        &self,
        commands: &mut [CommandItem],
        context: &context::Context,
        _settings: &AppSettings,
    ) {
        // Boost "New Note" if text is selected
        if context.selected_text.is_some() {
            for cmd in commands.iter_mut() {
                if cmd.id == "note_new" {
                    // Custom ranking logic
                }
            }
        }
    }
}

#[tauri::command]
pub async fn save_note(
    content: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Save note to file or database
    Ok(())
}

#[tauri::command]
pub async fn get_notes(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    // Load notes from storage
    Ok(vec![])
}
```

---

## Testing Checklist

After implementing your feature, test the following:

### Backend Testing
- [ ] Feature appears in Command Palette
- [ ] Widget command opens widget window (if applicable)
- [ ] Action commands execute correctly (if applicable)
- [ ] Command handlers return expected data types
- [ ] Error handling works correctly
- [ ] Window appears over fullscreen apps (if widget)

### Frontend Testing
- [ ] Widget component renders correctly
- [ ] API calls work as expected
- [ ] State management works correctly
- [ ] UI is responsive and accessible
- [ ] Error states are handled gracefully

### Integration Testing
- [ ] Feature integrates with Command Palette search
- [ ] Context-aware ranking works (if implemented)
- [ ] Settings persistence works (if applicable)
- [ ] Window lifecycle (show/hide) works correctly

### Edge Cases
- [ ] Handles empty/null inputs
- [ ] Handles network errors (if applicable)
- [ ] Handles permission errors (if applicable)
- [ ] Works in fullscreen mode
- [ ] Works with multiple monitors

---

## Common Pitfalls

1. **Forgetting to register commands**: Always add commands to `invoke_handler` in `lib.rs`
2. **Missing window dimensions**: Add widget dimensions in both window creation functions
3. **Type mismatches**: Ensure Rust and TypeScript types match exactly
4. **Missing feature registration**: Add feature to `get_all_features()` in `features.rs`
5. **ActionType not added**: Add variants to `ActionType` enum if using actions
6. **Window routing**: Update `App.tsx` to route to your widget component

---

## Getting Help

If you encounter issues:

1. Check existing feature implementations (Translator, Currency, Clipboard)
2. Review the project structure in `PROJECT_STRUCTURE.md`
3. Check Tauri v2 documentation for IPC patterns
4. Review React 19 documentation for component patterns

---

## Next Steps

After implementing your feature:

1. Update `PROJECT_STRUCTURE.md` to include your new files
2. Add feature documentation to `README.md`
3. Consider adding unit tests
4. Update changelog/release notes

