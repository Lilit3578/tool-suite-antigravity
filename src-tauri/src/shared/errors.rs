//! Strict error handling with CommandError enum
//!
//! Replaces all `Result<T, String>` with proper error types using thiserror.
//! All errors are serializable for IPC communication.

use thiserror::Error;
use serde::Serialize;

/// Command execution errors
///
/// This enum provides strict error handling for all command operations.
/// All variants are serializable for IPC communication with the frontend.
#[derive(Error, Debug, Clone, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    /// System I/O error (file operations, network, etc.)
    #[error("System I/O error: {0}")]
    SystemIO(String),

    /// Mathematical calculation error
    #[error("Math error: {0}")]
    MathError(String),

    /// Required feature is missing or unavailable
    #[error("Feature missing: {0}")]
    FeatureMissing(String),

    /// Accessibility permissions denied
    #[error("Accessibility permissions denied. Please enable in System Settings > Privacy & Security > Accessibility.")]
    AccessibilityDenied,

    /// Invalid input or parameter
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Window operation error
    #[error("Window error: {0}")]
    WindowError(String),

    /// Clipboard operation error
    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    /// Network/API error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Unknown/unexpected error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Implement From for common error types
impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::SystemIO(err.to_string())
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(err: reqwest::Error) -> Self {
        CommandError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for CommandError {
    fn from(err: serde_json::Error) -> Self {
        CommandError::InvalidInput(format!("JSON error: {}", err))
    }
}

// Conversion from legacy AppError for gradual migration
impl From<crate::shared::error::AppError> for CommandError {
    fn from(err: crate::shared::error::AppError) -> Self {
        match err {
            crate::shared::error::AppError::Io(msg) => CommandError::SystemIO(msg),
            crate::shared::error::AppError::Network(msg) => CommandError::NetworkError(msg),
            crate::shared::error::AppError::System(msg) => CommandError::SystemIO(msg),
            crate::shared::error::AppError::Calculation(msg) => CommandError::MathError(msg),
            crate::shared::error::AppError::Validation(msg) => CommandError::InvalidInput(msg),
            crate::shared::error::AppError::Clipboard(msg) => CommandError::ClipboardError(msg),
            crate::shared::error::AppError::Feature(msg) => CommandError::FeatureMissing(msg),
            crate::shared::error::AppError::Unknown(msg) => CommandError::Unknown(msg),
        }
    }
}

// Helper type alias for command results
pub type CommandResult<T> = Result<T, CommandError>;

// Error message constants (for backward compatibility)
pub const ERR_UNSUPPORTED_ACTION: &str = "Unsupported action type";
