# macOS Productivity Widgets

A polished macOS-only desktop utility that gives users quick access to small productivity widgets (text translation, currency conversion, command palette) from a global hotkey or menubar.

## 🎯 Project Status

**Phase 1 & 2 Complete** ✅

- ✅ Tauri v2 project scaffolded with React + TypeScript + Vite
- ✅ All dependencies installed (Rust + Frontend)
- ✅ macOS-specific Tauri configuration
- ✅ System tray/menubar integration
- ✅ Global shortcut manager with error handling
- ✅ Settings persistence (JSON file-based)
- ✅ All IPC command stubs implemented
- ✅ Minimal test frontend

## 🏗️ Architecture

### Backend (Rust/Tauri)
- **Settings Management**: Persistent storage in `~/.config/productivity-widgets/settings.json`
- **Global Shortcuts**: Configurable hotkeys for each widget
- **Tray Menu**: System tray icon with menu items for all widgets
- **IPC Commands**: Type-safe communication between frontend and backend

### Frontend (React + TypeScript)
- **Vite**: Fast development server and build tool
- **Tailwind CSS**: Utility-first CSS framework
- **Lucide React**: Icon library
- **Fuse.js**: Fuzzy search for command palette

## 🚀 Getting Started

### Prerequisites
- macOS 10.15 or later
- Node.js 18+ and npm
- Rust and Cargo (install via [rustup](https://rustup.rs/))
- Xcode Command Line Tools

### Development Mode

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

The app will start with:
- A tray icon in the macOS menu bar
- Global shortcuts registered (if not conflicting with system shortcuts)
- Console output showing initialization status

### Building for Production

```bash
# Build the app
npm run tauri build
```

This creates a `.dmg` file in `src-tauri/target/release/bundle/dmg/`.

## ⌨️ Default Hotkeys

| Widget | Hotkey | Description |
|--------|--------|-------------|
| Command Palette | `Cmd+Shift+P` | Open quick command palette |
| Translator | `Cmd+Shift+T` | Open translation widget |
| Currency Converter | `Cmd+Shift+C` | Open currency converter |

**Note**: If a hotkey fails to register (already in use), you can:
1. Change it in the settings file
2. Use the tray menu to open widgets instead

## 📁 Project Structure

```
productivity-widgets/
├── src/                          # Frontend React app
│   ├── App.tsx                   # Main app component with widget routing
│   ├── main.tsx                  # React entry point
│   └── index.css                 # Tailwind CSS styles
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Main Tauri app setup
│   │   ├── main.rs               # Entry point
│   │   ├── settings.rs           # Settings persistence module
│   │   ├── types.rs              # IPC type definitions
│   │   └── commands.rs           # IPC command implementations
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Node dependencies
└── tailwind.config.js            # Tailwind configuration
```

## 🔧 Configuration

### Settings File Location
`~/.config/productivity-widgets/settings.json`

### Settings Structure
```json
{
  "hotkeys": {
    "command_palette": "CmdOrCtrl+Shift+P",
    "translator": "CmdOrCtrl+Shift+T",
    "currency_converter": "CmdOrCtrl+Shift+C"
  },
  "api_keys": {
    "translation_provider": "google",
    "translation_key": "",
    "currency_api_key": ""
  },
  "preferences": {
    "default_source_lang": "auto",
    "default_target_lang": "en",
    "default_currency_from": "USD",
    "default_currency_to": "EUR",
    "theme": "system"
  }
}
```

## 🧪 Testing Phase 1 & 2

1. **Start the app**: `npm run tauri dev`
2. **Verify tray icon**: Look for the app icon in the macOS menu bar
3. **Test tray menu**: Click the tray icon and select a widget
4. **Test IPC commands**: Click the test buttons in the widget windows
5. **Check console output**: Look for "✅ Productivity Widgets initialized successfully!"

### Expected Console Output
```
✅ Registered global shortcut for command palette: CmdOrCtrl+Shift+P
✅ Registered global shortcut for translator: CmdOrCtrl+Shift+T
✅ Registered global shortcut for currency converter: CmdOrCtrl+Shift+C
✅ Productivity Widgets initialized successfully!
📋 Hotkeys configured:
   Command Palette: CmdOrCtrl+Shift+P
🌐 Translator: CmdOrCtrl+Shift+T
💱 Currency Converter: CmdOrCtrl+Shift+C
```

## 🔌 IPC API

### Available Commands

#### `get_settings() -> AppSettings`
Get current application settings.

#### `save_settings(settings: AppSettings) -> ()`
Save application settings.

#### `capture_selection(mode?: string) -> CaptureResult`
Capture text from selection or clipboard.
- **Stub**: Returns empty text (Phase 5 implementation)

#### `translate_text(request: TranslateRequest) -> TranslateResponse`
Translate text using configured API.
- **Stub**: Returns mock translation (Phase 4 implementation)

#### `convert_currency(request: ConvertCurrencyRequest) -> ConvertCurrencyResponse`
Convert between currencies.
- **Stub**: Returns mock conversion with fixed rate (Phase 4 implementation)

#### `log_message(request: LogRequest) -> ()`
Log a message to console.

## 🛠️ Development Roadmap

### ✅ Phase 1: Foundation & Scaffold (Complete)
- Tauri v2 project setup
- Dependencies installed
- Tailwind CSS configured
- macOS-specific configuration

### ✅ Phase 2: Backend Core (Complete)
- System tray/menubar
- Global shortcuts with error handling
- Settings persistence
- IPC command stubs

### 🔄 Phase 3: Frontend Architecture (Next)
- UI state management
- Command palette component
- Fuzzy search integration
- Keyboard navigation

### 📋 Phase 4: Widget Implementation
- Translation API integration
- Currency conversion API
- Widget UIs

### 🖥️ Phase 5: OS Integration
- Clipboard capture
- Accessibility API for selection capture
- Permission handling

### 📦 Phase 6: Packaging & Polish
- Code signing
- Notarization
- DMG creation
- Final documentation

## 🐛 Known Issues

1. **Global Shortcuts May Fail**: If system shortcuts conflict, registration will fail gracefully. Use tray menu as fallback.
2. **Unused Type Warning**: `OpenWidgetRequest` struct is defined for future use.
3. **Deprecated Config Value**: Minor warning in `tauri.conf.json` (line 28) - does not affect functionality.

## 📝 License

This project is created for demonstration purposes.

## 🤝 Contributing

This is a demonstration project for implementing a macOS productivity tool with Tauri v2.

---

**Built with**: Tauri v2, React, TypeScript, Rust, Tailwind CSS
