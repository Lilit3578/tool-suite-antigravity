//! Clipboard module
//!
//! Provides clipboard history tracking and monitoring functionality.
//!
//! This module contains two main components:
//! - `history`: Manages clipboard history with deduplication and capacity limits
//! - `monitor`: Background thread that monitors clipboard changes

pub mod history;
pub mod monitor;

pub use history::{ClipboardHistory, ClipboardItem};
pub use monitor::ClipboardMonitor;
