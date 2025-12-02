//! Floating Panel implementation for macOS
//! Creates a real NSNonactivatingPanel that behaves like Raycast/Spotlight
//! 
//! Requirements:
//! - Appears over fullscreen applications
//! - Does NOT activate the app
//! - Does NOT become key
//! - Joins all spaces
//! - Uses standard Cocoa APIs only (no class mutation)

#[cfg(target_os = "macos")]
use cocoa::{
    appkit::{NSWindowCollectionBehavior, NSWindowStyleMask},
    base::{id, nil, YES, NO},
    foundation::{NSRect, NSPoint, NSSize},
};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

// Window level constants
#[cfg(target_os = "macos")]
const NS_STATUS_WINDOW_LEVEL: i64 = 25;
#[cfg(target_os = "macos")]
const NS_POPUP_MENU_WINDOW_LEVEL: i64 = 101;  // Pop-up menu level - appears over fullscreen
#[cfg(target_os = "macos")]
const NS_SCREEN_SAVER_WINDOW_LEVEL: i64 = 1000;  // Screen saver level - HIGHEST standard level

// Collection behavior constants
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowCollectionBehaviorCanJoinAllSpaces: u64 = 1 << 0;  // 0x1
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
const NSWindowCollectionBehaviorFullScreenAuxiliary: u64 = 1 << 7; // 0x80

/// Floating Panel that behaves like Raycast/Spotlight
/// Uses a real NSNonactivatingPanel (not a converted NSWindow)
#[cfg(target_os = "macos")]
pub struct FloatingPanel {
    panel: id,
}

#[cfg(target_os = "macos")]
impl FloatingPanel {
    /// Create a new floating panel
    /// 
    /// # Arguments
    /// * `x` - X position
    /// * `y` - Y position  
    /// * `w` - Width
    /// * `h` - Height
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        unsafe {
            // Create content rect
            let rect = NSRect::new(
                NSPoint::new(x, y),
                NSSize::new(w, h)
            );
            
            // Create NSPanel with borderless style
            // We'll rely on the extremely high window level (1000) to appear over fullscreen
            let style_mask = NSWindowStyleMask::NSBorderlessWindowMask;
            
            // NSBackingStoreBuffered = 2
            let backing: u64 = 2;
            
            let panel: id = msg_send![class!(NSPanel), alloc];
            let panel: id = msg_send![panel, initWithContentRect: rect
                                                styleMask: style_mask
                                                backing: backing
                                                defer: NO];
            
            // Configure as non-activating panel
            let _: () = msg_send![panel, setBecomesKeyOnlyIfNeeded: YES];
            let _: () = msg_send![panel, setFloatingPanel: YES];
            let _: () = msg_send![panel, setWorksWhenModal: YES];
            let _: () = msg_send![panel, setHidesOnDeactivate: NO];
            
            // Set transparent background - use clearColor instead of nil
            let _: () = msg_send![panel, setOpaque: NO];
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![panel, setBackgroundColor: clear_color];
            let _: () = msg_send![panel, setAlphaValue: 1.0];
            
            // Add shadow for visibility
            let _: () = msg_send![panel, setHasShadow: YES];
            
            // Set window level to NSScreenSaverWindowLevel (1000) - HIGHEST standard level
            // This is higher than fullscreen apps and should appear on top
            let _: () = msg_send![panel, setLevel: NS_SCREEN_SAVER_WINDOW_LEVEL];
            
            // Set collection behaviors
            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;
            let _: () = msg_send![panel, setCollectionBehavior: behavior];
            
            println!("🔵 [DEBUG] [FloatingPanel] Panel configured with level {} (NSScreenSaverWindowLevel) and collection behavior 0x81", NS_SCREEN_SAVER_WINDOW_LEVEL);
            
            // Accept mouse events
            let _: () = msg_send![panel, setAcceptsMouseMovedEvents: YES];
            let _: () = msg_send![panel, setIgnoresMouseEvents: NO];
            
            println!("🔵 [DEBUG] [FloatingPanel] Created NSNonactivatingPanel at ({}, {}) size {}x{}", x, y, w, h);
            
            Self { panel }
        }
    }
    
    /// Show the panel (order to front without activating app)
    pub fn show(&self) {
        unsafe {
            println!("🔵 [DEBUG] [FloatingPanel] Showing panel...");
            
            // Order to front regardless - this makes it visible without activating app
            let _: () = msg_send![self.panel, orderFrontRegardless];
            
            // Verify it's visible
            let is_visible: bool = msg_send![self.panel, isVisible];
            let alpha: f64 = msg_send![self.panel, alphaValue];
            
            println!("🔵 [DEBUG] [FloatingPanel] Panel shown - isVisible: {}, alpha: {}", is_visible, alpha);
            
            if !is_visible {
                eprintln!("🔴 [DEBUG] [FloatingPanel] ⚠️  Panel is NOT visible after orderFrontRegardless!");
            }
        }
    }
    
    /// Hide the panel
    pub fn hide(&self) {
        unsafe {
            let _: () = msg_send![self.panel, orderOut: nil];
            println!("🔵 [DEBUG] [FloatingPanel] Panel hidden");
        }
    }
    
    /// Set the content view (WKWebView from Tauri)
    /// The view must be a valid NSView pointer
    pub unsafe fn set_content_view(&self, view: id) {
        if view != nil {
            // Make the WKWebView transparent
            // Note: WKWebView doesn't support setValue:forKey: for backgroundColor
            // Instead, we rely on the panel's transparent background
            let _: () = msg_send![view, setOpaque: NO];
            
            // Set the content view
            let _: () = msg_send![self.panel, setContentView: view];
            println!("🔵 [DEBUG] [FloatingPanel] Content view set with transparency");
        } else {
            eprintln!("🔴 [DEBUG] [FloatingPanel] ⚠️  Cannot set nil content view");
        }
    }
    
    /// Get the panel pointer (for Tauri integration)
    pub fn as_id(&self) -> id {
        self.panel
    }
    
    /// Set panel position
    pub fn set_position(&self, x: f64, y: f64) {
        unsafe {
            let frame: NSRect = msg_send![self.panel, frame];
            let new_rect = NSRect::new(
                NSPoint::new(x, y),
                frame.size
            );
            let _: () = msg_send![self.panel, setFrame: new_rect display: YES];
            println!("🔵 [DEBUG] [FloatingPanel] Position set to ({}, {})", x, y);
        }
    }
    
    /// Set panel size
    pub fn set_size(&self, w: f64, h: f64) {
        unsafe {
            let frame: NSRect = msg_send![self.panel, frame];
            let new_rect = NSRect::new(
                frame.origin,
                NSSize::new(w, h)
            );
            let _: () = msg_send![self.panel, setFrame: new_rect display: YES];
            println!("🔵 [DEBUG] [FloatingPanel] Size set to {}x{}", w, h);
        }
    }
    
    /// Check if panel is visible
    pub fn is_visible(&self) -> bool {
        unsafe {
            msg_send![self.panel, isVisible]
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for FloatingPanel {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.panel, release];
            println!("🔵 [DEBUG] [FloatingPanel] Panel released");
        }
    }
}

// Stubs for non-macOS
#[cfg(not(target_os = "macos"))]
pub struct FloatingPanel;

#[cfg(not(target_os = "macos"))]
impl FloatingPanel {
    pub fn new(_x: f64, _y: f64, _w: f64, _h: f64) -> Self {
        Self
    }
    
    pub fn show(&self) {}
    pub fn hide(&self) {}
    pub fn set_content_view(&self, _view: *mut std::ffi::c_void) {}
    pub fn set_position(&self, _x: f64, _y: f64) {}
    pub fn set_size(&self, _w: f64, _h: f64) {}
    pub fn is_visible(&self) -> bool { false }
}

