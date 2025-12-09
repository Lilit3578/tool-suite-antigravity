pub mod types;
pub mod settings;
pub mod errors;
pub mod error;
pub mod events;
pub mod emit;

#[cfg(test)]
mod types_test;

// Re-export CommandError for convenience
pub use errors::{CommandError, CommandResult};
