//! Command modules for Tauri application
//!
//! This module organizes commands into feature-specific submodules following
//! Modern Rust idioms (no mod.rs pattern).
//!
//! ## Architecture
//!
//! - `palette`: Command palette core logic (simplified)
//! - `window`: Window positioning and management
//! - `system`: System integration (accessibility, logging)
//! - `settings`: Settings persistence
//!
//! Feature-specific commands (translator, currency, clipboard) have been
//! moved to the `features` module for better scalability.
//! Error handling is in the parent `api` module.

// Core command modules
pub mod palette;
pub mod window;
pub mod system;
pub mod settings;

