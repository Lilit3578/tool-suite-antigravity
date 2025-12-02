//! Centralized error handling utilities for command modules
//!
//! This module provides consistent error formatting across all Tauri commands.

use std::fmt;

/// Format an API error with service name and error details
#[allow(dead_code)]
pub fn format_api_error(service: &str, error: &str) -> String {
    format!("{} API error: {}", service, error)
}

/// Format an I/O error
#[allow(dead_code)]
pub fn format_io_error(operation: &str, error: &std::io::Error) -> String {
    format!("Failed to {}: {}", operation, error)
}

/// Format a parse error
#[allow(dead_code)]
pub fn format_parse_error(what: &str, error: &str) -> String {
    format!("Failed to parse {}: {}", what, error)
}

/// Format a clipboard error
#[allow(dead_code)]
pub fn format_clipboard_error(operation: &str, error: &str) -> String {
    format!("Clipboard {}: {}", operation, error)
}

/// Format a window management error
pub fn format_window_error(operation: &str, error: &str) -> String {
    format!("Window {}: {}", operation, error)
}

/// Trait for converting errors into command-friendly error strings
#[allow(dead_code)]
pub trait IntoCommandError {
    fn into_command_error(self) -> String;
}

impl<E: fmt::Display> IntoCommandError for E {
    fn into_command_error(self) -> String {
        self.to_string()
    }
}

/// Result type alias for Tauri commands
pub type CommandResult<T> = Result<T, String>;
