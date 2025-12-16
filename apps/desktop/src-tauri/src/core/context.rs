//! Context detection and ranking module
//!
//! Provides intelligent context detection (language, currency) and
//! usage-based command ranking for the command palette.

pub mod detection;
pub mod ranking;
pub mod category;
pub mod validation;

pub use detection::{detect_currency, detect_language};
pub use ranking::{UsageMetrics, rank_commands, score_by_context};
pub use category::{ContextCategory, detect_content_category, get_action_category, get_widget_category};
pub use validation::validate_action;
