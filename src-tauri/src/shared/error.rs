use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug, Serialize)]
pub enum AppError {
    #[error("I/O Error: {0}")]
    Io(String),

    #[error("Network Error: {0}")]
    Network(String),

    #[error("System Error: {0}")]
    System(String),

    #[error("Calculation Error: {0}")]
    Calculation(String),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Clipboard Error: {0}")]
    Clipboard(String),

    #[error("Feature Error: {0}")]
    Feature(String),

    #[error("Unknown Error: {0}")]
    Unknown(String),
}

// Implement conversion from standard errors
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Validation(format!("Serialization error: {}", err))
    }
}

// Convert string errors (legacy support during refactor)
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Unknown(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Unknown(err.to_string())
    }
}

// Convert CommandError to AppError for gradual migration
impl From<crate::shared::errors::CommandError> for AppError {
    fn from(err: crate::shared::errors::CommandError) -> Self {
        match err {
            crate::shared::errors::CommandError::SystemIO(msg) => AppError::Io(msg),
            crate::shared::errors::CommandError::NetworkError(msg) => AppError::Network(msg),
            crate::shared::errors::CommandError::MathError(msg) => AppError::Calculation(msg),
            crate::shared::errors::CommandError::InvalidInput(msg) => AppError::Validation(msg),
            crate::shared::errors::CommandError::ClipboardError(msg) => AppError::Clipboard(msg),
            crate::shared::errors::CommandError::FeatureMissing(msg) => AppError::Feature(msg),
            crate::shared::errors::CommandError::AccessibilityDenied => AppError::System("Accessibility permissions denied".to_string()),
            crate::shared::errors::CommandError::WindowError(msg) => AppError::System(msg),
            crate::shared::errors::CommandError::Unknown(msg) => AppError::Unknown(msg),
        }
    }
}

// Helper for Tauri Result
pub type AppResult<T> = Result<T, AppError>;
