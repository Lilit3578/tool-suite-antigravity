---
trigger: always_on
---

Productivity Widgets - macOS Desktop Utility
Overview
Productivity Widgets is a polished macOS-only desktop application that provides quick access to a suite of productivity tools through a global hotkey or system tray menu. It's built as a Tauri desktop application (Rust + React) and runs in the macOS menu bar, offering lightweight, always-accessible utilities without consuming prominent screen space.

Goal & Purpose
The application aims to maximize user productivity by providing instant access to commonly-needed utilities while minimizing disruption to workflow. Users can invoke powerful tools with a single keyboard shortcut or tray click, without launching full applications or context-switching. This follows the philosophy of keeping high-value tools "at your fingertips."

Key Features & Functionality
1. Command Palette (Primary Access Point)
Hotkey: Control+Shift+L
A floating, transparent command palette that appears at the cursor location
Automatically captures selected text from any application by simulating Cmd+C
Fuzzy search across all available commands and widgets
Keyboard-navigable with arrow keys and Enter to execute
Non-resizable, stays on top of other windows
Intelligently positions itself within screen boundaries
2. Translator Widget
Translate text between languages in real-time
Features dropdown selectors for source and target languages
Auto-detection of source language available
Planned integration with translation APIs (Google Translate, etc.)
3. Currency Converter Widget
Convert amounts between different currencies
Dropdown selectors for source/target currencies
Real-time exchange rate calculation
Planned integration with live currency exchange APIs
4. Clipboard History
Maintains a history of clipboard entries (capacity: ~5 items)
Tracks clipboard monitor status (enabled/paused)
Quick access to recently copied items via tray menu
Ability to clear history or toggle monitoring from tray
Intelligent ranking: Uses usage metrics to prioritize frequently used items
5. Settings Widget
Configure hotkeys for each widget
Manage API keys for external services (translation, currency)
Store language and currency preferences
Theme selection (system, light, dark)
Persistent JSON-based storage at ~/.config/productivity-widgets/settings.json
6. System Tray Integration
Always-accessible menu bar icon on macOS
Quick menu options:
Open Command Palette
Clipboard History (shows count of items)
Pause/Resume clipboard monitoring
Clear clipboard history
Settings
Quit application

How It Works
Architecture Overview
Workflow Example: Using the Translator
User presses Control+Shift+L anywhere on macOS
Rust backend captures active app context and simulates Cmd+C to copy selected text
Command Palette opens near the cursor with clipboard text pre-populated
User types to search or selects "Translate" from palette
Translator widget opens with text already in the input field
User selects source/target languages and clicks translate
Backend translates text via configured API
Result displays instantly in the output field
User can copy result and close widget with one click
Key Technical Details
Global Shortcut System: Tauri's global shortcut plugin monitors OS-level keypresses independent of application focus
Clipboard Monitoring: Background thread continuously monitors macOS clipboard for changes, stores history with timestamps
Intelligent Context Detection: Records command usage and workspace context for smarter command ranking
macOS-Specific Features: Uses Tauri's macOS Private API for enhanced integration (tray icons, positioning, system integration)
IPC Communication: React components communicate with Rust backend via strongly-typed Tauri commands
Persistent State: Settings stored in JSON format, survives app restarts

Component Breakdown

Component
Purpose
CommandPalette.tsx
Fuzzy-searchable command interface with keyboard navigation
TranslatorWidget.tsx
Text translation UI with language selectors
CurrencyConverterWidget.tsx
Currency conversion with live exchange rates
ClipboardHistoryWidget.tsx
Browse and reuse clipboard entries
SettingsWidget.tsx
Configure hotkeys, API keys, and preferences
lib/utils.ts
Utility functions for UI rendering
store.ts
Zustand state management for all widget states
api.ts
Wrapper around Tauri IPC commands
lib.rs
Main Tauri app initialization, tray setup, shortcuts
settings.rs
Settings file I/O and persistence
clipboard/mod.rs
Clipboard history management
clipboard/monitor.rs
Background thread monitoring clipboard
automation/macos.rs
macOS-specific automations (Cmd+C, active app detection)
context/ranking.rs
Intelligent ranking based on usage patterns



Technology Stack
Layer
Technology
Desktop Framework
Tauri 2
Frontend
React 19, TypeScript, 
Styling
Tailwind CSS, Radix UI components
State Management
Zustand
Search/Fuzzy Matching
Fuse.js
Backend
Rust, async-std
IPC/Commands
Tauri invoke handler
System Integration
macOS Private APIs, global shortcuts, clipboard monitor


Use Cases
Quick Translation: Highlight text in any app → Control+Shift+L → search "translate" → done
Currency Lookup: Check conversion rates during browsing without opening calculator
Clipboard Management: Recover previously copied text without digging through history
Cross-App Command Execution: Execute actions from one central command center
Workflow Automation: Recurring tasks accessible via consistent hotkey across all apps

This is a sophisticated, modern productivity tool designed for power users who value speed and minimal friction in their workflows.

