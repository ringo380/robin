// Native macOS window implementation using Cocoa
// Provides real NSWindow with CAMetalLayer for Metal rendering

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSEvent, NSEventMask,
    NSEventType, NSRunningApplication, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::runtime::{Object, Class};
use objc::{msg_send, sel, sel_impl};
use core_graphics::geometry::{CGPoint, CGSize};

use std::collections::HashSet;
use std::sync::Mutex;
use crate::logging::{LogCategory, log_info, log_debug};

// Global state for window delegate
static mut WINDOW_SHOULD_CLOSE: bool = false;
use std::sync::LazyLock;
static KEYS_PRESSED: LazyLock<Mutex<HashSet<u16>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static MOUSE_DELTA: LazyLock<Mutex<CGPoint>> = LazyLock::new(|| Mutex::new(CGPoint { x: 0.0, y: 0.0 }));

pub struct NativeWindow {
    _pool: id, // NSAutoreleasePool as id
    window: id,
    view: id,
    metal_layer: id,
    size: CGSize,
    mouse_grabbed: bool,
}

impl NativeWindow {
    pub fn new(title: &str, width: f64, height: f64) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            // Initialize NSApplication
            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);

            // Create window
            let window_rect = NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(width, height));
            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                window_rect,
                NSWindowStyleMask::NSTitledWindowMask
                    | NSWindowStyleMask::NSClosableWindowMask
                    | NSWindowStyleMask::NSResizableWindowMask
                    | NSWindowStyleMask::NSMiniaturizableWindowMask,
                NSBackingStoreType::NSBackingStoreBuffered,
                NO,
            );

            // Set window title
            let title_ns = NSString::alloc(nil).init_str(title);
            window.setTitle_(title_ns);

            // Create Metal view
            let view = NSView::alloc(nil).initWithFrame_(window_rect);
            window.setContentView_(view);

            // Enable layer hosting
            let _: () = msg_send![view, setWantsLayer: YES];

            // Make view accept key input to prevent system beeping
            let _: () = msg_send![view, setAcceptsFirstResponder: YES];

            // Create CAMetalLayer with enhanced error handling and architecture detection
            let metal_layer = Self::create_metal_layer_safe()?;

            // Set the layer on the view
            let _: () = msg_send![view, setLayer: metal_layer];

            // Configure Metal layer for proper rendering with error checking
            Self::configure_metal_layer_safe(metal_layer, width, height)?;
            let drawable_size = CGSize::new(width, height);
            let _: () = msg_send![metal_layer, setDrawableSize: drawable_size];

            // Note: Window delegate setup would go here if needed
            // For simplicity, we handle window closing through the global flag

            // Center and show window
            window.center();
            window.makeKeyAndOrderFront_(nil);

            // Force window to front
            window.orderFrontRegardless();
            window.makeMainWindow();
            window.makeKeyWindow();

            // Activate application with force
            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(cocoa::appkit::NSApplicationActivateIgnoringOtherApps);

            // Force app to front
            app.activateIgnoringOtherApps_(YES);

            // Make the view the first responder to handle keyboard input without beeping
            let _: () = msg_send![window, makeFirstResponder: view];

            log_info!(LogCategory::Window, "Real macOS window created with Metal layer");
            log_debug!(LogCategory::Window, "Window forced to foreground and activated");
            log_debug!(LogCategory::Window, "View set as first responder for input handling");

            Ok(Self {
                _pool: pool,
                window,
                view,
                metal_layer,
                size: CGSize::new(width, height),
                mouse_grabbed: false,
            })
        }
    }

    /// Creates a Metal layer with enhanced error handling and architecture detection
    unsafe fn create_metal_layer_safe() -> Result<id, Box<dyn std::error::Error>> {
        // Check if we're running under Rosetta translation
        if Self::is_running_under_rosetta() {
            log_info!(LogCategory::Window, "⚠️  WARNING: Running under Rosetta translation");
            log_info!(LogCategory::Window, "ℹ️  For optimal performance, build with: cargo build --target aarch64-apple-darwin");
        }

        // Verify Metal framework is available
        let metal_layer_class = Class::get("CAMetalLayer")
            .ok_or("CAMetalLayer class not found - Metal framework may not be available")?;

        log_debug!(LogCategory::Window, "✅ CAMetalLayer class loaded successfully");

        // Create Metal layer with error checking
        let metal_layer: id = msg_send![metal_layer_class, layer];
        if metal_layer == nil {
            return Err("Failed to create CAMetalLayer instance".into());
        }

        log_debug!(LogCategory::Window, "✅ CAMetalLayer instance created successfully");
        Ok(metal_layer)
    }

    /// Configures Metal layer with safer initialization and validation
    unsafe fn configure_metal_layer_safe(metal_layer: id, width: f64, height: f64) -> Result<(), Box<dyn std::error::Error>> {
        // Set pixel format with validation
        let pixel_format: u64 = 80; // BGRA8Unorm
        let _: () = msg_send![metal_layer, setPixelFormat: pixel_format];

        // Verify pixel format was set correctly
        let actual_format: u64 = msg_send![metal_layer, pixelFormat];
        if actual_format != pixel_format {
            log_debug!(LogCategory::Window, "⚠️  Pixel format mismatch: expected {}, got {}", pixel_format, actual_format);
        }

        // Configure layer properties
        let _: () = msg_send![metal_layer, setOpaque: YES];
        let _: () = msg_send![metal_layer, setPresentsWithTransaction: NO];

        // Set initial drawable size
        let drawable_size = CGSize::new(width, height);
        let _: () = msg_send![metal_layer, setDrawableSize: drawable_size];

        log_debug!(LogCategory::Window, "✅ Metal layer configured successfully ({}x{})", width, height);
        Ok(())
    }

    /// Detects if the application is running under Rosetta translation
    fn is_running_under_rosetta() -> bool {
        unsafe {
            let mut ret = 0i32;
            let mut size = std::mem::size_of::<i32>();
            let result = libc::sysctlbyname(
                b"sysctl.proc_translated\0".as_ptr() as *const i8,
                &mut ret as *mut _ as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );
            result == 0 && ret == 1
        }
    }

    pub fn get_metal_layer(&self) -> id {
        self.metal_layer
    }

    pub fn get_view(&self) -> &Object {
        unsafe { &*(self.view as *const Object) }
    }

    pub fn get_size(&self) -> CGSize {
        self.size
    }

    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        let mut events = Vec::new();

        unsafe {
            // Process all available events
            loop {
                let event = NSApp().nextEventMatchingMask_untilDate_inMode_dequeue_(
                    NSEventMask::NSAnyEventMask.bits(),
                    nil,
                    cocoa::foundation::NSDefaultRunLoopMode,
                    YES,
                );

                if event == nil {
                    break;
                }

                let event_type = event.eventType();

                match event_type {
                    NSEventType::NSKeyDown => {
                        let key_code = event.keyCode();
                        if let Ok(mut keys) = KEYS_PRESSED.lock() {
                            keys.insert(key_code);
                        }
                        events.push(WindowEvent::KeyPressed(key_code));
                    }
                    NSEventType::NSKeyUp => {
                        let key_code = event.keyCode();
                        if let Ok(mut keys) = KEYS_PRESSED.lock() {
                            keys.remove(&key_code);
                        }
                        events.push(WindowEvent::KeyReleased(key_code));
                    }
                    NSEventType::NSMouseMoved => {
                        if self.mouse_grabbed {
                            let delta_x = event.deltaX();
                            let delta_y = event.deltaY();
                            let delta = CGPoint::new(delta_x, delta_y);
                            if let Ok(mut mouse_delta) = MOUSE_DELTA.lock() {
                                *mouse_delta = delta;
                            }
                            events.push(WindowEvent::MouseMoved(delta));
                        }
                    }
                    NSEventType::NSLeftMouseDown => {
                        events.push(WindowEvent::MousePressed(MouseButton::Left));
                    }
                    NSEventType::NSLeftMouseUp => {
                        events.push(WindowEvent::MouseReleased(MouseButton::Left));
                    }
                    NSEventType::NSRightMouseDown => {
                        events.push(WindowEvent::MousePressed(MouseButton::Right));
                    }
                    NSEventType::NSRightMouseUp => {
                        events.push(WindowEvent::MouseReleased(MouseButton::Right));
                    }
                    NSEventType::NSOtherMouseDown => {
                        if event.buttonNumber() == 2 {
                            self.toggle_mouse_grab();
                            events.push(WindowEvent::MousePressed(MouseButton::Middle));
                        }
                    }
                    _ => {}
                }

                // Send event to application
                NSApp().sendEvent_(event);
            }

            // Check for window close
            if WINDOW_SHOULD_CLOSE {
                events.push(WindowEvent::WindowClosed);
            }
        }

        events
    }

    pub fn is_key_pressed(&self, key_code: u16) -> bool {
        if let Ok(keys) = KEYS_PRESSED.lock() {
            keys.contains(&key_code)
        } else {
            false
        }
    }

    pub fn toggle_mouse_grab(&mut self) {
        self.mouse_grabbed = !self.mouse_grabbed;

        // TODO: Implement cursor hide/show when needed

        log_debug!(LogCategory::Input, "Mouse grab: {}", if self.mouse_grabbed { "ON" } else { "OFF" });
    }

    pub fn should_close(&self) -> bool {
        unsafe { WINDOW_SHOULD_CLOSE }
    }

    pub fn update_size(&mut self) {
        unsafe {
            let content_rect = NSView::frame(self.view);
            self.size = CGSize::new(content_rect.size.width, content_rect.size.height);

            // Update Metal layer drawable size
            let _: () = msg_send![self.metal_layer, setDrawableSize: self.size];
        }
    }
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self._pool, drain];
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    KeyPressed(u16),
    KeyReleased(u16),
    MousePressed(MouseButton),
    MouseReleased(MouseButton),
    MouseMoved(CGPoint),
    WindowResized(CGSize),
    WindowClosed,
}

#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

// macOS key codes
pub mod key_codes {
    pub const W: u16 = 13;
    pub const A: u16 = 0;
    pub const S: u16 = 1;
    pub const D: u16 = 2;
    pub const SPACE: u16 = 49;
    pub const LEFT_SHIFT: u16 = 56;
    pub const RIGHT_SHIFT: u16 = 60;
    pub const B: u16 = 11;
    pub const T: u16 = 17;
    pub const P: u16 = 35;
    pub const G: u16 = 5;
    pub const C: u16 = 8;
    pub const V: u16 = 9;
    pub const R: u16 = 15;
    pub const Z: u16 = 6;
    pub const Y: u16 = 16;
    pub const TAB: u16 = 48;
    pub const KEY_1: u16 = 18;
    pub const KEY_2: u16 = 19;
    pub const KEY_3: u16 = 20;
    pub const KEY_4: u16 = 21;
    pub const KEY_5: u16 = 23;
    pub const KEY_6: u16 = 22;
    pub const KEY_7: u16 = 26;
    pub const KEY_8: u16 = 28;
    pub const KEY_9: u16 = 25;
    pub const ESCAPE: u16 = 53;
    pub const CMD: u16 = 55;
    pub const F: u16 = 3;
    // Function keys for demo mode switching
    pub const F1: u16 = 122;
    pub const F2: u16 = 120;
    pub const F3: u16 = 99;
    pub const F4: u16 = 118;
    pub const F5: u16 = 96;
    pub const F6: u16 = 97;
}