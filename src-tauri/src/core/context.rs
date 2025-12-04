//! Context detection and ranking module
//!
//! Provides intelligent context detection (language, currency) and
//! usage-based command ranking for the command palette.

pub mod detection;
pub mod ranking;

pub use detection::{detect_currency, detect_language};
pub use ranking::{UsageMetrics, rank_commands};
