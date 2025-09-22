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

            // Create CAMetalLayer
            let metal_layer_class = Class::get("CAMetalLayer").ok_or("CAMetalLayer class not found")?;
            let metal_layer: id = msg_send![metal_layer_class, layer];

            // Set the layer on the view
            let _: () = msg_send![view, setLayer: metal_layer];

            // Configure Metal layer
            let _: () = msg_send![metal_layer, setPixelFormat: 80]; // BGRA8Unorm
            let drawable_size = CGSize::new(width, height);
            let _: () = msg_send![metal_layer, setDrawableSize: drawable_size];

            // Note: Window delegate setup would go here if needed
            // For simplicity, we handle window closing through the global flag

            // Center and show window
            window.center();
            window.makeKeyAndOrderFront_(nil);

            // Activate application
            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(cocoa::appkit::NSApplicationActivateIgnoringOtherApps);

            println!("✅ Real macOS window created with Metal layer");

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

        println!("🖱️  Mouse grab: {}", if self.mouse_grabbed { "ON" } else { "OFF" });
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
}