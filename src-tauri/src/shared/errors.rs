//! Error message constants
//!
//! Centralized error messages for consistency and future internationalization.

pub const ERR_MISSING_TEXT_PARAM: &str = "Missing 'text' parameter";
pub const ERR_NEGATIVE_LENGTH: &str = "Length cannot be negative. Please provide a positive value.";
pub const ERR_NEGATIVE_MASS: &str = "Mass cannot be negative. Please provide a positive value.";
pub const ERR_NEGATIVE_VOLUME: &str = "Volume cannot be negative. Please provide a positive value.";
pub const ERR_UNSUPPORTED_ACTION: &str = "Unsupported action type";
pub const ERR_CANNOT_PARSE_UNIT: &str = "Could not parse unit from text";
pub const ERR_WINDOW_HANDLE_UNAVAILABLE: &str = "Window handle not available";

