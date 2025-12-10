use std::sync::{Arc, atomic::AtomicBool};

/// Thread-safe clipboard state
#[derive(Clone)]
pub struct ClipboardState {
    /// Flag to ignore the next clipboard change event
    /// Used to prevent "ghost" copies (app-initiated copies) from polluting history
    pub ignore_next: Arc<AtomicBool>,
}

impl ClipboardState {
    pub fn new() -> Self {
        Self {
            ignore_next: Arc::new(AtomicBool::new(false)),
        }
    }
}
