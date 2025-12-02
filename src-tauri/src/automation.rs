//! Automation module for macOS system integration
//!
//! Provides automation capabilities for clipboard operations, focus management,
//! and accessibility features.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::{
    auto_paste_flow,
    get_active_app,
    restore_focus,
    simulate_cmd_c,
    detect_text_selection, // ← NEW: Smart selection detection
};

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn get_active_app() -> Result<String, String> {
    Err("Not supported on this platform".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_cmd_c() -> Result<(), String> {
    Err("Not supported on this platform".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn restore_focus(_app: &str) -> Result<(), String> {
    Err("Not supported on this platform".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn auto_paste_flow(_app: &str, _delay_ms: u64) -> Result<(), String> {
    Err("Not supported on this platform".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permissions() -> bool {
    true
}
