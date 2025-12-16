//! Window Handle Management
//! 
//! Provides safe Objective-C object handle management for macOS windows.
//! This module ensures proper retain/release semantics and prevents memory leaks.

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::runtime::{objc_retain, objc_release};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// SafeObjcHandle - Memory-safe wrapper for Objective-C objects
// ============================================================================

/// Safe wrapper for Objective-C object pointers
/// 
/// This struct ensures that Objective-C objects are properly retained when
/// stored in Rust and released when dropped, preventing use-after-free bugs.
/// 
/// # Safety
/// - The wrapped pointer must be a valid Objective-C object
/// - The object will be retained on creation and released on drop
/// - This struct is Send + Sync because Objective-C objects are thread-safe
///   when accessed via message sending (which is atomic)
#[cfg(target_os = "macos")]
pub struct SafeObjcHandle {
    ptr: id,
}

#[cfg(target_os = "macos")]
unsafe impl Send for SafeObjcHandle {}

#[cfg(target_os = "macos")]
unsafe impl Sync for SafeObjcHandle {}

#[cfg(target_os = "macos")]
impl SafeObjcHandle {
    /// Create a new safe handle from an Objective-C object pointer
    /// 
    /// # Arguments
    /// * `ptr` - The Objective-C object pointer (id)
    /// 
    /// # Returns
    /// * `Some(SafeObjcHandle)` if pointer is non-null
    /// * `None` if pointer is null
    /// 
    /// # Safety
    /// The pointer must be a valid Objective-C object. This function will
    /// retain the object, so the caller must ensure the object is not
    /// deallocated before this handle is dropped.
    pub unsafe fn new(ptr: id) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        
        // Retain the object to prevent deallocation while Rust holds a reference
        let _: id = objc_retain(ptr);
        
        Some(Self { ptr })
    }

    /// Get the raw Objective-C object pointer
    /// 
    /// # Returns
    /// The wrapped Objective-C object pointer (id)
    /// 
    /// # Safety
    /// The returned pointer is valid as long as this handle exists.
    /// Do not release the object manually - the handle manages its lifetime.
    pub fn as_id(&self) -> id {
        self.ptr
    }
}

#[cfg(target_os = "macos")]
impl Drop for SafeObjcHandle {
    fn drop(&mut self) {
        // Release the object when the handle is dropped
        unsafe {
            objc_release(self.ptr);
        }
    }
}

// Stub implementation for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub struct SafeObjcHandle;

#[cfg(not(target_os = "macos"))]
impl SafeObjcHandle {
    pub unsafe fn new(_ptr: *const std::ffi::c_void) -> Option<Self> {
        None
    }
    
    pub fn as_id(&self) -> *const std::ffi::c_void {
        std::ptr::null()
    }
}

// ============================================================================
// WindowHandleManager - Manages SafeObjcHandle instances
// ============================================================================

/// Manages SafeObjcHandle instances for all windows
/// 
/// This manager ensures that Objective-C window pointers are properly
/// retained and released, preventing memory leaks and use-after-free bugs.
#[derive(Clone)]
pub struct WindowHandleManager {
    #[cfg(target_os = "macos")]
    handles: Arc<Mutex<HashMap<String, Arc<SafeObjcHandle>>>>,
    #[cfg(not(target_os = "macos"))]
    handles: Arc<Mutex<HashMap<String, ()>>>,
}

impl WindowHandleManager {
    /// Create a new window handle manager
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a window handle
    /// 
    /// # Arguments
    /// * `window_label` - The Tauri window label (e.g., "palette-window")
    /// * `handle` - The SafeObjcHandle for the window
    /// 
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` if registration fails
    #[cfg(target_os = "macos")]
    pub fn register(&self, window_label: &str, handle: Arc<SafeObjcHandle>) -> Result<(), String> {
        let mut handles = self.handles.lock()
            .map_err(|e| format!("Failed to lock handle manager: {}", e))?;
        
        let label = window_label.to_string();
        handles.insert(label.clone(), handle);
        println!("[WindowHandleManager] Registered handle for window: {}", label);
        Ok(())
    }

    /// Get a window handle by label
    /// 
    /// # Arguments
    /// * `window_label` - The Tauri window label
    /// 
    /// # Returns
    /// * `Some(Arc<SafeObjcHandle>)` if handle exists
    /// * `None` if handle not found
    #[cfg(target_os = "macos")]
    pub fn get(&self, window_label: &str) -> Option<Arc<SafeObjcHandle>> {
        let handles = self.handles.lock().ok()?;
        handles.get(window_label).cloned()
    }

    // Stub implementations for non-macOS platforms
    #[cfg(not(target_os = "macos"))]
    pub fn register(&self, _window_label: &str, _handle: ()) -> Result<(), String> {
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn get(&self, _window_label: &str) -> Option<()> {
        None
    }
}

impl Default for WindowHandleManager {
    fn default() -> Self {
        Self::new()
    }
}
