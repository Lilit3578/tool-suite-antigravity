# Project Structure

## File Architecture

```
tool-suite-antigravity/
├── components.json                    # shadcn/ui component configuration
├── index.html                         # Main HTML entry point
├── package.json                       # Node.js dependencies and scripts
├── package-lock.json                  # Locked dependency versions
├── postcss.config.js                 # PostCSS configuration for Tailwind
├── README.md                          # Project documentation
├── tailwind.config.js                 # Tailwind CSS configuration
├── tsconfig.json                      # TypeScript configuration
├── tsconfig.node.json                 # TypeScript config for Node.js tools
├── vite.config.ts                     # Vite build tool configuration
│
├── public/                             # Static assets served directly
│   ├── tauri.svg                      # Tauri logo
│   └── vite.svg                       # Vite logo
│
├── src/                               # Frontend React application
│   ├── main.tsx                       # React application entry point
│   ├── App.tsx                        # Root React component, window routing
│   │
│   ├── assets/                        # Static frontend assets
│   │   └── react.svg
│   │
│   ├── components/                    # React components
│   │   ├── CommandPalette.tsx        # Main command palette UI component
│   │   │                              # - Handles search, filtering, command execution
│   │   │                              # - Manages widget window lifecycle
│   │   │
│   │   ├── widgets/                   # Feature widget components
│   │   │   ├── ClipboardHistoryWidget.tsx    # Clipboard history display & management
│   │   │   ├── CurrencyConverterWidget.tsx    # Currency conversion interface
│   │   │   ├── SettingsWidget.tsx             # Application settings UI
│   │   │   └── TranslatorWidget.tsx          # Text translation interface
│   │   │
│   │   └── ui/                        # shadcn/ui primitive components
│   │       ├── badge.tsx              # Badge component
│   │       ├── button.tsx             # Button component
│   │       ├── card.tsx               # Card container component
│   │       ├── combobox.tsx           # Combobox/autocomplete component
│   │       ├── command.tsx            # Command palette UI primitives
│   │       ├── dialog.tsx             # Modal dialog component
│   │       ├── input.tsx              # Text input component
│   │       ├── label.tsx              # Form label component
│   │       ├── popover.tsx            # Popover component
│   │       ├── select.tsx             # Select dropdown component
│   │       ├── separator.tsx          # Visual separator component
│   │       ├── tabs.tsx               # Tabs component
│   │       └── textarea.tsx           # Textarea component
│   │
│   ├── logic/                         # Frontend business logic
│   │   ├── api/
│   │   │   └── tauri.ts               # Tauri IPC API client wrapper
│   │   │                              # - Type-safe command invocations
│   │   │                              # - Event listeners
│   │   │
│   │   ├── state/
│   │   │   └── store.ts               # Global state management (Zustand/Jotai)
│   │   │                              # - Settings state
│   │   │                              # - Clipboard history state
│   │   │                              # - UI state
│   │   │
│   │   ├── types/
│   │   │   ├── index.ts               # TypeScript type definitions
│   │   │   │                          # - Command types
│   │   │   │                          # - Widget types
│   │   │   │                          # - Settings types
│   │   │   └── vite-env.d.ts          # Vite environment type definitions
│   │   │
│   │   └── utils/
│   │       └── helpers.ts             # Utility functions
│   │                                  # - Text formatting
│   │                                  # - Data transformations
│   │
│   └── styles/                        # CSS stylesheets
│       ├── App.css                    # Application-specific styles
│       └── index.css                  # Global styles and Tailwind imports
│
├── src-tauri/                         # Tauri backend (Rust)
│   ├── build.rs                       # Build script for native dependencies
│   ├── Cargo.toml                     # Rust dependencies and project metadata
│   ├── Cargo.lock                     # Locked Rust dependency versions
│   ├── tauri.conf.json                # Tauri application configuration
│   │                                 # - Window settings
│   │                                 # - Security policies
│   │                                 # - Bundle configuration
│   │
│   ├── capabilities/                  # Tauri v2 capability definitions
│   │   └── default.json               # Default capability manifest
│   │
│   ├── gen/                           # Generated files (build artifacts)
│   │   └── schemas/                   # Tauri schema definitions
│   │       ├── acl-manifests.json     # Access control list manifests
│   │       ├── capabilities.json      # Capability definitions
│   │       ├── desktop-schema.json    # Desktop platform schema
│   │       └── macOS-schema.json       # macOS-specific schema
│   │
│   ├── icons/                         # Application icons
│   │   ├── icon.icns                  # macOS icon bundle
│   │   ├── icon.ico                   # Windows icon
│   │   ├── icon.png                   # Default icon
│   │   └── [various sizes].png        # Platform-specific icon sizes
│   │
│   └── src/                           # Rust source code
│       ├── main.rs                    # Application entry point
│       │                              # - Initializes Tauri app
│       │                              # - Sets up global shortcuts
│       │                              # - Configures system tray
│       │
│       ├── lib.rs                     # Main library module
│       │                              # - Application setup
│       │                              # - Window management
│       │                              # - Global shortcut handlers
│       │                              # - Widget window lifecycle
│       │
│       ├── api.rs                     # API module root
│       │
│       ├── api/                       # Command API layer
│       │   ├── commands.rs            # Command module root
│       │   ├── error.rs               # Error types and handling
│       │   │
│       │   └── commands/              # Individual command handlers
│       │       ├── palette.rs          # Command palette commands
│       │       │                      # - capture_selection()
│       │       │                      # - get_command_items()
│       │       │                      # - execute_action()
│       │       │                      # - record_command_usage()
│       │       │
│       │       ├── settings.rs         # Settings management commands
│       │       │                      # - get_settings()
│       │       │                      # - save_settings()
│       │       │
│       │       ├── system.rs           # System integration commands
│       │       │                      # - get_active_app()
│       │       │                      # - check_accessibility_permissions()
│       │       │                      # - log_message()
│       │       │
│       │       └── window.rs          # Window management commands
│       │                              # - get_cursor_position()
│       │                              # - get_primary_monitor_bounds()
│       │                              # - calculate_palette_position()
│       │                              # - hide_palette_window()
│       │                              # - show_widget()
│       │
│       ├── core.rs                    # Core module root
│       │
│       ├── core/                      # Core business logic
│       │   ├── clipboard.rs           # Clipboard module root
│       │   │
│       │   ├── clipboard/             # Clipboard functionality
│       │   │   ├── history.rs          # Clipboard history storage
│       │   │   │                      # - History item management
│       │   │   │                      # - Persistence (optional)
│       │   │   │
│       │   │   └── monitor.rs          # Clipboard monitoring
│       │   │                          # - Background clipboard watcher
│       │   │                          # - Change detection
│       │   │                          # - History updates
│       │   │
│       │   ├── context.rs             # Context module root
│       │   │
│       │   ├── context/               # Context detection & ranking
│       │   │   ├── detection.rs        # Context detection logic
│       │   │   │                      # - Language detection
│       │   │   │                      # - Selection detection
│       │   │   │
│       │   │   └── ranking.rs          # Command ranking algorithm
│       │   │                          # - Relevance scoring
│       │   │                          # - Usage-based ranking
│       │   │
│       │   ├── features.rs            # Features module root
│       │   │
│       │   └── features/              # Feature implementations
│       │       ├── clipboard.rs        # Clipboard feature commands
│       │       │                      # - get_clipboard_history()
│       │       │                      # - paste_clipboard_item()
│       │       │                      # - clear_clipboard_history()
│       │       │                      # - toggle_clipboard_monitor()
│       │       │
│       │       ├── currency.rs         # Currency conversion feature
│       │       │                      # - convert_currency()
│       │       │                      # - Exchange rate fetching
│       │       │
│       │       └── translator.rs       # Translation feature
│       │                              # - translate_text()
│       │                              # - Language detection
│       │                              # - Translation API integration
│       │
│       ├── shared.rs                  # Shared module root
│       │
│       ├── shared/                    # Shared types and utilities
│       │   ├── settings.rs            # Settings data structures
│       │   │                          # - AppSettings struct
│       │   │                          # - Settings persistence
│       │   │                          # - JSON serialization
│       │   │
│       │   └── types.rs               # Shared type definitions
│       │                              # - Common data structures
│       │                              # - Error types
│       │
│       ├── system.rs                  # System module root
│       │
│       └── system/                    # System integration layer
│           ├── automation.rs          # Automation module root
│           │
│           ├── automation/            # System automation
│           │   └── macos.rs           # macOS-specific automation
│           │                          # - Accessibility API integration
│           │                          # - Text selection simulation
│           │                          # - App switching
│           │                          # - Permission checks
│           │
│           ├── window.rs              # Window module root
│           │
│           └── window/                # Window management
│               ├── nswindow.rs        # macOS NSWindow integration
│               │                     # - Window level configuration
│               │                     # - Collection behavior (Spaces)
│               │                     # - Fullscreen overlay support
│               │                     # - Non-activating window display
│               │
│               └── panel.rs           # NSPanel configuration
│                                   # - Panel window setup
│                                   # - Non-activating panel behavior
│
├── target/                            # Rust build artifacts (generated)
│   └── [debug|release]/               # Debug and release builds
│
├── dist/                              # Frontend build output (generated)
│   └── assets/                        # Bundled frontend assets
│
└── node_modules/                      # Node.js dependencies (generated)
```

## Architecture Overview

### High-Level Architecture

- **Frontend (Renderer)**: React 19 + TypeScript + Vite
  - Renders UI components and widgets
  - Manages client-side state
  - Handles user interactions

- **Backend (Core)**: Rust + Tauri v2
  - System integration (tray, shortcuts, clipboard, automation)
  - Window management and positioning
  - Data persistence
  - Native macOS APIs

- **IPC Communication**: Tauri `invoke()` / `emit()`
  - Request/response pattern for commands
  - Event-based communication for real-time updates

### Key Components

#### Frontend
- **CommandPalette.tsx**: Main UI component for command search and execution
- **Widgets**: Feature-specific UI components (Translator, Currency, Clipboard, Settings)
- **API Client** (`logic/api/tauri.ts`): Type-safe Tauri command invocations
- **State Management** (`logic/state/store.ts`): Global application state

#### Backend
- **lib.rs**: Application initialization, window lifecycle, global shortcuts
- **Commands** (`api/commands/`): IPC command handlers
- **Core Features** (`core/features/`): Business logic for translator, currency, clipboard
- **System Integration** (`system/`): macOS automation, window management, NSWindow configuration
- **Clipboard** (`core/clipboard/`): History storage and background monitoring

### Data Flow

1. **User triggers shortcut** → Backend global shortcut handler
2. **Backend captures selection** (optional) → Shows palette window
3. **Frontend loads** → Requests settings and clipboard history
4. **User searches/selects** → Frontend invokes backend commands
5. **Backend executes** → Returns results or opens widget windows

### Window Management

- **Palette Window**: Transient, transparent, always-on-top, positioned near cursor
- **Widget Windows**: Standard windows, may hide on blur
- **Fullscreen Overlay**: Uses `NSWindowLevel` 25 and `CanJoinAllSpaces` collection behavior to appear over fullscreen apps without activating

### Persistence

- **Settings**: JSON file stored via `shared/settings.rs`
- **Clipboard History**: In-memory structure (optional persistence)
- **Command Usage**: Tracked for ranking and suggestions

### Security & Permissions

- macOS Accessibility/Automation permissions required for:
  - Text selection detection
  - Clipboard monitoring
  - App switching
- Tauri capabilities restrict IPC access
- CSP policies restrict frontend resource loading

---

# Project Structure - Productivity Widgets

*** End Patch

| Path | Purpose |
|------|---------|
| `src/main.tsx` | React application entry point |
| `src/App.tsx` | Main application component, routing logic |
| `src/store.ts` | Zustand state management (widget states) |
| `src/api.ts` | Tauri IPC command wrappers |
| `src/components/CommandPalette.tsx` | Main command palette interface |
| `src/components/TranslatorWidget.tsx` | Translation widget UI |
| `src/components/CurrencyConverterWidget.tsx` | Currency conversion UI |
| `src/components/ClipboardHistoryWidget.tsx` | Clipboard history browser |
| `src/components/SettingsWidget.tsx` | Settings configuration UI |
| `src/components/ui/*` | Reusable Shadcn UI components |

### Backend (Rust + Tauri)

| Path | Purpose |
|------|---------|
| **Core** | |
| `src-tauri/src/lib.rs` | Main app setup, global shortcuts, tray menu, Accessory mode |
| `src-tauri/src/main.rs` | Binary entry point |
| `src-tauri/src/types.rs` | Shared type definitions |
| `src-tauri/src/settings.rs` | Settings file I/O (~/.config/productivity-widgets/settings.json) |
| **Window Management** | |
| `src-tauri/src/nswindow.rs` | ⭐ **macOS window configuration for fullscreen overlay** |
| `src-tauri/src/panel.rs` | NSPanel implementation (floating panels) |
| **Commands** | |
| `src-tauri/src/commands/palette.rs` | Command palette backend logic |
| `src-tauri/src/commands/window.rs` | Window positioning & management |
| `src-tauri/src/commands/settings.rs` | Settings CRUD operations |
| `src-tauri/src/commands/system.rs` | System utilities (active app, permissions) |
| **Features** | |
| `src-tauri/src/features/translator.rs` | Translation API integration |
| `src-tauri/src/features/currency.rs` | Currency conversion API |
| `src-tauri/src/features/clipboard.rs` | Clipboard history commands |
| **Clipboard** | |
| `src-tauri/src/clipboard/history.rs` | Clipboard history storage (in-memory) |
| `src-tauri/src/clipboard/monitor.rs` | Background thread monitoring clipboard |
| **Automation** | |
| `src-tauri/src/automation/macos.rs` | macOS automation (Cmd+C simulation, active app detection) |
| **Context** | |
| `src-tauri/src/context/detection.rs` | Context analysis (language detection, etc.) |
| `src-tauri/src/context/ranking.rs` | Usage metrics for intelligent command ranking |

### Configuration Files

| File | Purpose |
|------|---------|
| `tauri.conf.json` | Tauri app configuration (bundle ID, permissions, window settings) |
| `Cargo.toml` | Rust dependencies (tauri, cocoa, objc, etc.) |
| `package.json` | Node.js dependencies (React, Vite, Tailwind, etc.) |
| `vite.config.ts` | Vite build configuration |
| `tailwind.config.js` | Tailwind CSS theme & plugins |
| `tsconfig.json` | TypeScript compiler options |
| `components.json` | Shadcn UI component configuration |

## Critical Implementation Files

### Fullscreen Overlay Implementation

The following files implement the fullscreen overlay functionality:

1. **[src-tauri/src/lib.rs](src-tauri/src/lib.rs#L27-L38)**
   - Sets app activation policy to Accessory mode
   - Prevents space-switching when app activates

2. **[src-tauri/src/nswindow.rs](src-tauri/src/nswindow.rs)**
   - `set_app_activation_policy_accessory()` - Sets Accessory mode
   - `show_window_over_fullscreen()` - Configures NSWindow with panel-like behavior
   - `configure_window_for_fullscreen()` - Sets window level & collection behavior
   - `run_on_main_thread()` - Ensures AppKit operations run on main thread

3. **[src-tauri/src/panel.rs](src-tauri/src/panel.rs)**
   - `FloatingPanel` struct - NSPanel wrapper (legacy, not currently used)

## Data Flow

```
User Triggers Shortcut (Control+Shift+Space)
    ↓
Global Shortcut Handler (lib.rs)
    ↓
Detect Text Selection (automation/macos.rs)
    ↓
Show Command Palette Window (lib.rs → nswindow.rs)
    ↓
Configure for Fullscreen Overlay (nswindow.rs)
    ↓
Display Window Over Fullscreen App ✅
    ↓
User Selects Command
    ↓
Execute Command (commands/palette.rs)
    ↓
Show Widget Window (translator, currency, etc.)
    ↓
Configure Widget for Fullscreen Overlay (nswindow.rs)
    ↓
Display Widget Over Fullscreen App ✅
```

## Build Artifacts

| Directory | Purpose |
|-----------|---------|
| `target/` | Rust build output (debug/release binaries) |
| `dist/` | Vite build output (bundled frontend) |
| `node_modules/` | Node.js dependencies |
| `src-tauri/target/` | Tauri build artifacts |

## Configuration Locations

| Type | Location |
|------|----------|
| User Settings | `~/.config/productivity-widgets/settings.json` |
| App Bundle | `target/release/bundle/macos/Productivity Widgets.app` |
| Development Binary | `target/debug/productivity-widgets` |

## Technology Stack

| Layer | Technologies |
|-------|-------------|
| **Frontend** | React 19, TypeScript, Tailwind CSS, Shadcn UI, Zustand |
| **Backend** | Rust, Tauri v2, Cocoa (macOS), Objc |
| **Build Tools** | Vite, Cargo, npm |
| **State Management** | Zustand (frontend), Arc<Mutex<T>> (backend) |
| **IPC** | Tauri invoke/emit system |
| **Styling** | Tailwind CSS, Radix UI primitives |

## Lines of Code (Approximate)

| Component | Files | Lines |
|-----------|-------|-------|
| Frontend (TypeScript/React) | 20+ | ~3,000 |
| Backend (Rust) | 25+ | ~4,500 |
| Configuration | 10+ | ~500 |
| **Total** | **55+** | **~8,000** |


----


